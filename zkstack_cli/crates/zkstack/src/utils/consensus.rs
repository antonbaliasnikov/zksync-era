use anyhow::Context as _;
use secrecy::{ExposeSecret, Secret};
use zkstack_cli_config::ChainConfig;
use zksync_config::configs::consensus::{
    AttesterPublicKey, AttesterSecretKey, ConsensusSecrets, GenesisSpec, NodePublicKey,
    NodeSecretKey, ProtocolVersion, ValidatorPublicKey, ValidatorSecretKey, WeightedAttester,
    WeightedValidator,
};
use zksync_consensus_crypto::{Text, TextFmt};
use zksync_consensus_roles::{attester, node, validator};

pub(crate) fn parse_attester_committee(
    attesters: &[WeightedAttester],
) -> anyhow::Result<attester::Committee> {
    let attesters: Vec<_> = attesters
        .iter()
        .enumerate()
        .map(|(i, v)| {
            Ok(attester::WeightedAttester {
                key: Text::new(&v.key.0).decode().context("key").context(i)?,
                weight: v.weight,
            })
        })
        .collect::<anyhow::Result<_>>()
        .context("attesters")?;
    attester::Committee::new(attesters).context("Committee::new()")
}

#[derive(Debug, Clone)]
pub struct ConsensusSecretKeys {
    validator_key: validator::SecretKey,
    attester_key: attester::SecretKey,
    node_key: node::SecretKey,
}

pub struct ConsensusPublicKeys {
    validator_key: validator::PublicKey,
    attester_key: attester::PublicKey,
}

pub fn generate_consensus_keys() -> ConsensusSecretKeys {
    ConsensusSecretKeys {
        validator_key: validator::SecretKey::generate(),
        attester_key: attester::SecretKey::generate(),
        node_key: node::SecretKey::generate(),
    }
}

fn get_consensus_public_keys(consensus_keys: &ConsensusSecretKeys) -> ConsensusPublicKeys {
    ConsensusPublicKeys {
        validator_key: consensus_keys.validator_key.public(),
        attester_key: consensus_keys.attester_key.public(),
    }
}

pub fn get_genesis_specs(
    chain_config: &ChainConfig,
    consensus_keys: &ConsensusSecretKeys,
) -> GenesisSpec {
    let public_keys = get_consensus_public_keys(consensus_keys);
    let validator_key = public_keys.validator_key.encode();
    let attester_key = public_keys.attester_key.encode();

    let validator = WeightedValidator {
        key: ValidatorPublicKey(validator_key.clone()),
        weight: 1,
    };
    let attester = WeightedAttester {
        key: AttesterPublicKey(attester_key),
        weight: 1,
    };
    let leader = ValidatorPublicKey(validator_key);

    GenesisSpec {
        chain_id: chain_config.chain_id,
        protocol_version: ProtocolVersion(1),
        validators: vec![validator],
        attesters: vec![attester],
        leader,
        registry_address: None,
        seed_peers: [].into(),
    }
}

pub fn get_consensus_secrets(consensus_keys: &ConsensusSecretKeys) -> ConsensusSecrets {
    let validator_key = consensus_keys.validator_key.encode();
    let attester_key = consensus_keys.attester_key.encode();
    let node_key = consensus_keys.node_key.encode();

    ConsensusSecrets {
        validator_key: Some(ValidatorSecretKey(Secret::new(validator_key))),
        attester_key: Some(AttesterSecretKey(Secret::new(attester_key))),
        node_key: Some(NodeSecretKey(Secret::new(node_key))),
    }
}

pub fn node_public_key(secrets: &ConsensusSecrets) -> anyhow::Result<Option<NodePublicKey>> {
    Ok(node_key(secrets)?.map(|node_secret_key| NodePublicKey(node_secret_key.public().encode())))
}
fn node_key(secrets: &ConsensusSecrets) -> anyhow::Result<Option<node::SecretKey>> {
    read_secret_text(secrets.node_key.as_ref().map(|x| &x.0))
}

fn read_secret_text<T: TextFmt>(text: Option<&Secret<String>>) -> anyhow::Result<Option<T>> {
    text.map(|text| Text::new(text.expose_secret()).decode())
        .transpose()
        .map_err(|_| anyhow::format_err!("invalid format"))
}

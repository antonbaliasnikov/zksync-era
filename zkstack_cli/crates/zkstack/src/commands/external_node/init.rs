use anyhow::Context;
use xshell::Shell;
use zkstack_cli_common::{
    db::{drop_db_if_exists, init_db, migrate_db, DatabaseConfig},
    spinner::Spinner,
};
use zkstack_cli_config::{
    traits::ReadConfigWithBasePath, ChainConfig, EcosystemConfig, SecretsConfig,
};

use crate::{
    consts::SERVER_MIGRATIONS,
    messages::{
        MSG_CHAIN_NOT_INITIALIZED, MSG_DATABASE_MUST_BE_PRESENTED,
        MSG_EXTERNAL_NODE_CONFIG_NOT_INITIALIZED, MSG_FAILED_TO_DROP_SERVER_DATABASE_ERR,
        MSG_INITIALIZING_DATABASES_SPINNER,
    },
    utils::rocks_db::{recreate_rocksdb_dirs, RocksDBDirOption},
};

pub async fn run(shell: &Shell) -> anyhow::Result<()> {
    let ecosystem_config = EcosystemConfig::from_file(shell)?;

    let chain_config = ecosystem_config
        .load_current_chain()
        .context(MSG_CHAIN_NOT_INITIALIZED)?;

    init(shell, &chain_config).await
}

pub async fn init(shell: &Shell, chain_config: &ChainConfig) -> anyhow::Result<()> {
    let spin = Spinner::new(MSG_INITIALIZING_DATABASES_SPINNER);
    let secrets = SecretsConfig::read_with_base_path(
        shell,
        chain_config
            .external_node_config_path
            .clone()
            .context(MSG_EXTERNAL_NODE_CONFIG_NOT_INITIALIZED)?,
    )?;
    let db_config = DatabaseConfig::from_url(
        secrets
            .database
            .as_ref()
            .context(MSG_DATABASE_MUST_BE_PRESENTED)?
            .master_url()?
            .expose_url(),
    )?;
    drop_db_if_exists(&db_config)
        .await
        .context(MSG_FAILED_TO_DROP_SERVER_DATABASE_ERR)?;
    init_db(&db_config).await?;
    recreate_rocksdb_dirs(
        shell,
        &chain_config.rocks_db_path,
        RocksDBDirOption::ExternalNode,
    )?;
    let path_to_server_migration = chain_config.link_to_code.join(SERVER_MIGRATIONS);
    migrate_db(shell, path_to_server_migration, &db_config.full_url()).await?;
    spin.finish();
    Ok(())
}

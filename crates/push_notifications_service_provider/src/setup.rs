use std::path::PathBuf;

use anyhow::anyhow;
use holochain::prelude::{DnaModifiersOpt, RoleSettings, RoleSettingsMap, YamlProperties};
use holochain_client::{AgentPubKey, ExternIO, ZomeCallTarget};
use holochain_runtime::HolochainRuntime;
use roles_types::Properties;

use crate::read_from_file;

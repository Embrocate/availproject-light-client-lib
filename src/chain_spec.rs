// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Substrate chain configuration.

// TODO: document all that correctly

use fnv::FnvBuildHasher;
use hashbrown::{HashMap, HashSet};
use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};

use primitive_types::{H256, U256};

// TODO: shouldn't be public
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(crate) struct ChildRawStorage {
    pub(crate) child_info: Vec<u8>,
    pub(crate) child_type: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
/// Storage content for genesis block.
struct RawGenesis {
    top: HashMap<StorageKey, StorageData, FnvBuildHasher>,
    children_default: HashMap<StorageKey, ChildRawStorage, FnvBuildHasher>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
enum Genesis {
    Raw(RawGenesis),
}

// TODO: shouldn't be public
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StorageKey(#[serde(with = "impl_serde::serialize")] pub(crate) Vec<u8>);

// TODO: shouldn't be public
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StorageData(#[serde(with = "impl_serde::serialize")] pub(crate) Vec<u8>);

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct StorageChangeSet<Hash> {
    /// Block hash
    block: Hash,
    /// A list of changes
    changes: Vec<(StorageKey, Option<StorageData>)>,
}

/// A configuration of a client. Does not include runtime storage initialization.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct ClientSpec {
    name: String,
    id: String,
	#[serde(default)]
	chain_type: ChainType,
    boot_nodes: Vec<String>,
    telemetry_endpoints: Option<TelemetryEndpoints>,
    protocol_id: Option<String>,
    properties: Option<Properties>,
	fork_blocks: Option<Vec<(u64, H256)>>,
    bad_blocks: Option<HashSet<H256>>,
    // Unused but for some reason still part of the chain specs.
    consensus_engine: (),
    // TODO: looks deprecated
    genesis: Genesis,
}

/// The type of a chain.
///
/// This can be used by tools to determine the type of a chain for displaying
/// additional information or enabling additional features.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ChainType {
	/// A development chain that runs mainly on one node.
	Development,
	/// A local chain that runs locally on multiple nodes for testing purposes.
	Local,
	/// A live chain.
	Live,
	/// Some custom chain type.
	Custom(String),
}

impl Default for ChainType {
	fn default() -> Self {
		Self::Live
	}
}

/// List of telemetry servers we want to talk to. Contains the URL of the server, and the
/// maximum verbosity level.
///
/// The URL string can be either a URL or a multiaddress.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TelemetryEndpoints(Vec<(String, u8)>);

/// Arbitrary properties defined in chain spec as a JSON object
pub type Properties = serde_json::map::Map<String, serde_json::Value>;

/// A configuration of a chain. Can be used to build a genesis block.
#[derive(Clone)]
pub struct ChainSpec {
    client_spec: ClientSpec,
}

impl ChainSpec {
    /// A list of bootnode addresses.
    pub fn boot_nodes(&self) -> &[String] {
        &self.client_spec.boot_nodes
    }

    /// Spec name.
    pub fn name(&self) -> &str {
        &self.client_spec.name
    }

    /// Spec id.
    pub fn id(&self) -> &str {
        &self.client_spec.id
    }

    /// Network protocol id.
    pub fn protocol_id(&self) -> Option<&str> {
        self.client_spec.protocol_id.as_ref().map(String::as_str)
    }

    /// Add a bootnode to the list.
    pub fn add_boot_node(&mut self, addr: Multiaddr) {
        self.client_spec.boot_nodes.push(addr.to_string())
    }

    // TODO: bad API
    pub(crate) fn genesis_top(&self) -> &HashMap<StorageKey, StorageData, FnvBuildHasher> {
        let Genesis::Raw(genesis) = &self.client_spec.genesis;
        &genesis.top
    }

    // TODO: bad API
    pub(crate) fn genesis_children(&self) -> &HashMap<StorageKey, ChildRawStorage, FnvBuildHasher> {
        let Genesis::Raw(genesis) = &self.client_spec.genesis;
        &genesis.children_default
    }

    /*/// Create hardcoded spec.
    pub fn from_genesis<F: Fn() -> G + 'static + Send + Sync>(
        name: &str,
        id: &str,
        constructor: F,
        boot_nodes: Vec<String>,
        telemetry_endpoints: Option<TelemetryEndpoints>,
        protocol_id: Option<&str>,
        properties: Option<Properties>,
    ) -> Self {
        let client_spec = ClientSpec {
            name: name.to_owned(),
            id: id.to_owned(),
            boot_nodes,
            telemetry_endpoints,
            protocol_id: protocol_id.map(str::to_owned),
            properties,
            extensions,
            genesis: Default::default(),
        };

        ChainSpec {
            client_spec,
            genesis: GenesisSource::Factory(Arc::new(constructor)),
        }
    }*/

    /// Parse json content into a `ChainSpec`
    pub fn from_json_bytes(json: impl AsRef<[u8]>) -> Result<Self, String> {
        let client_spec = serde_json::from_slice(json.as_ref())
            .map_err(|e| format!("Error parsing spec file: {}", e))?;
        Ok(ChainSpec { client_spec })
    }
}

/*impl ChainSpec {
    /// Dump to json string.
    pub fn to_json(self, raw: bool) -> Result<String, String> {
        #[derive(Serialize, Deserialize)]
        struct Container<G, E> {
            #[serde(flatten)]
            client_spec: ClientSpec<E>,
            genesis: Genesis<G>,
        };

        let genesis = match (raw, self.genesis.resolve()?) {
            (true, Genesis::Runtime(g)) => {
                let storage = g.build_storage()?;
                let top = storage.top.into_iter()
                    .map(|(k, v)| (StorageKey(k), StorageData(v)))
                    .collect();
                let children = storage.children.into_iter()
                    .map(|(sk, child)| {
                        let info = child.child_info.as_ref();
                        let (info, ci_type) = info.info();
                        (
                            StorageKey(sk),
                            ChildRawStorage {
                                data: child.data.into_iter()
                                    .map(|(k, v)| (StorageKey(k), StorageData(v)))
                                    .collect(),
                                child_info: info.to_vec(),
                                child_type: ci_type,
                            },
                    )})
                    .collect();

                Genesis::Raw(RawGenesis { top, children })
            },
            (_, genesis) => genesis,
        };
        let container = Container {
            client_spec: self.client_spec,
            genesis,
        };
        serde_json::to_string_pretty(&container)
            .map_err(|e| format!("Error generating spec json: {}", e))
    }
}*/

#[cfg(test)]
mod tests {
    use super::ChainSpec;

    #[test]
    fn can_decode_polkadot_genesis() {
        // TODO: test not passing
        let spec = &include_bytes!("chain_spec/polkadot.json")[..];
        ChainSpec::from_json_bytes(&spec).unwrap();
    }
}
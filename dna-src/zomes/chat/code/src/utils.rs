use hdk::{
    error::{ZomeApiError, ZomeApiResult},
    holochain_core_types::{
        entry::{AppEntryValue, Entry},
        link::LinkMatch,
    },
    holochain_json_api::{
        json::{default_to_json, JsonString},
    },
    holochain_persistence_api::{
        cas::content::{ Address, AddressableContent },
    },
};
use serde::Serialize;
use std::{convert::TryFrom, fmt::Debug};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetLinksLoadResult<T> {
    pub entry: T,
    pub address: Address,
}

impl<T: Into<JsonString> + Debug + Serialize> From<GetLinksLoadResult<T>> for JsonString {
    fn from(u: GetLinksLoadResult<T>) -> JsonString {
        default_to_json(u)
    }
}

///
/// Helper function that perfoms a try_from for every entry
/// of a get_links_and_load for a given type. Any entries that either fail to
/// load or cannot be converted to the type will be dropped.
///
pub fn get_links_and_load_type<R: TryFrom<AppEntryValue>>(
    base: &Address,
    link_type: LinkMatch<&str>,
    tag: LinkMatch<&str>,
) -> ZomeApiResult<Vec<GetLinksLoadResult<R>>> {
    let link_load_results = hdk::get_links_and_load(base, link_type, tag)?;

    Ok(link_load_results
        .iter()
        .map(|maybe_entry| match maybe_entry {
            Ok(entry) => match entry {
                Entry::App(_, entry_value) => {
                    let typed_entry = R::try_from(entry_value.to_owned()).map_err(|_| {
                        ZomeApiError::Internal(
                            "Could not convert get_links result to requested type".to_string(),
                        )
                    })?;
                    Ok((entry.address(), typed_entry))
                }
                _ => Err(ZomeApiError::Internal(
                    "get_links did not return an app entry".to_string(),
                )),
            },
            _ => Err(ZomeApiError::Internal(
                "get_links did not return an app entry".to_string(),
            )),
        })
        .filter_map(Result::ok)
        .map(|(address, entry)| GetLinksLoadResult { address, entry })
        .collect())
}

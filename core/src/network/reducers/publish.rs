use holochain_net_connection::protocol_wrapper::DhtData;
use crate::agent::chain_header;
use crate::action::ActionWrapper;
use crate::network::state::NetworkState;
use holochain_net_connection::net_connection::NetConnection;
use crate::context::Context;
use std::sync::Arc;
use std::convert::TryInto;
use holochain_core_types::entry::SerializedEntry;
use holochain_core_types::error::HolochainError;
use holochain_core_types::entry::Entry;
use holochain_core_types::cas::content::AddressableContent;
use holochain_core_types::cas::content::Address;
use holochain_net_connection::protocol_wrapper::ProtocolWrapper;
use crate::network::actions::ActionResponse;

fn entry_from_cas(address: &Address, context: &Arc<Context>,) -> Result<Entry, HolochainError>{
    let json = context.file_storage.read()?.fetch(address)?
        .ok_or("Entry not found".to_string())?;
    let s: SerializedEntry = json.try_into()?;
    Ok(s.into())
}

pub fn reduce_publish(
    context: Arc<Context>,
    state: &mut NetworkState,
    action_wrapper: &ActionWrapper,
) {

    if state.network.is_none() || state.dna_hash.is_none() ||  state.agent_id.is_none() {
        return;
    }

    let action = action_wrapper.action();
    let address = unwrap_to!(action => crate::action::Action::Publish);

    let result = entry_from_cas(address, &context);
    if result.is_err() {
        return;
    };

    let (entry, maybe_header) = result.map(|entry|{
            let header = chain_header(&entry, &context);
            (entry, header)
        })
        .unwrap();

    if maybe_header.is_none() {
        // We don't have the entry in our source chain?!
        // Don't publish
        return;
    }

    //let header = maybe_header.unwrap();
    let data = DhtData {
        msg_id: "?".to_string(),
        dna_hash: state.dna_hash.clone().unwrap(),
        agent_id: state.agent_id.clone().unwrap(),
        address: entry.address().to_string(),
        content: serde_json::from_str(&entry.content().to_string()).unwrap(),
    };

    let response = match state.network {
        None => unreachable!(),
        Some(ref network) => {
            network.lock()
                .unwrap()
                .send(ProtocolWrapper::PublishDht(data).into())
        }
    };

    state.actions.insert(action_wrapper.clone(), ActionResponse::Publish(match response {
        Ok(_) => Ok(entry.address().to_owned()),
        Err(e) => Err(HolochainError::ErrorGeneric(e.to_string())),
    }));

}
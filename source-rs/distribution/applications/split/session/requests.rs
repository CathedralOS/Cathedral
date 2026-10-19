//! Authenticate owner/child streams and dispatch one event. No PID is trusted.
use super::*;
pub fn dispatch(
    event: Event,
    output: &mut Output,
    world: &mut World,
    children: &mut Children,
    server: &mut Server,
    survey: &mut Option<loss::Survey>,
    storage: &mut storage::Storage,
) -> Result<(), Error> {
    match event {
        Event::Disconnected { client, .. } => {
            if client >= 2 {
                storage.revoke(client - 2);
                server.remove(client);
                return Ok(());
            }
            storage.revoke(client);
            world.disconnected(client);
            output.flush(world)?;
            children.stop(client, server)?;
        }
        Event::Memory(request, object) => {
            if request.client.is_some_and(|c| c >= 2) {
                return storage.start(request, Some(object), server);
            }
            let reply = output.image(world, &request, object)?;
            server.reply(&request, &wire::encode(reply))?;
        }
        Event::Request(request) => {
            if request.client.is_some_and(|c| c >= 2) {
                return storage.start(request, None, server);
            }
            let words = wire::decode(&request.bytes[..request.len]);
            if request.control
                && words == Some([wire::LOSS, 0, 0, 0, 0, 0, 0, 0])
                && survey.is_none()
            {
                *survey = Some(loss::Survey::start(Some(request), children)?);
                return Ok(());
            }
            if !request.control && words.is_some_and(|w| w[0] == services::ACQUIRE) {
                let child = children.slots[request.client.unwrap()].as_mut().unwrap();
                if child.data.0.raw() == request.incarnation {
                    child
                        .services
                        .as_mut()
                        .unwrap()
                        .reply(server, &request, words.unwrap())?;
                    return Ok(());
                }
            }
            let reply = match words {
                Some([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]) if request.control => [0; 8],
                _ if request.control => wire::error(abi::DENIED),
                Some(words)
                    if matches!(words[0], wire::SURFACE_CREATE | wire::PAINT | wire::ERASE) =>
                {
                    output.surface(world, &request, words)?
                }
                Some(words) => world.request(request.client.unwrap(), request.incarnation, words),
                None => wire::error(abi::INVALID_ARGUMENT),
            };
            server.reply(&request, &wire::encode(reply))?;
        }
    }
    Ok(())
}

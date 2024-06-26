use crate::client::io::{ClientIoEventReceiver, ClientNetworkEventSender};
use crate::transport::dummy::DummyIo;
use crate::transport::error::Result;
use crate::transport::io::IoState;
use crate::transport::local::{LocalChannel, LocalChannelBuilder};
#[cfg(not(target_family = "wasm"))]
use crate::transport::udp::{UdpSocket, UdpSocketBuilder};
#[cfg(feature = "websocket")]
use crate::transport::websocket::client::{WebSocketClientSocket, WebSocketClientSocketBuilder};
#[cfg(feature = "webtransport")]
use crate::transport::webtransport::client::{
    WebTransportClientSocket, WebTransportClientSocketBuilder,
};
use enum_dispatch::enum_dispatch;

/// Transport combines a PacketSender and a PacketReceiver
///
/// This trait is used to abstract the raw transport layer that sends and receives packets.
/// There are multiple implementations of this trait, such as UdpSocket, WebSocket, WebTransport, etc.
#[enum_dispatch]
pub(crate) trait ClientTransportBuilder: Send + Sync {
    /// Attempt to connect to the remote
    fn connect(
        self,
    ) -> Result<(
        ClientTransportEnum,
        IoState,
        Option<ClientIoEventReceiver>,
        Option<ClientNetworkEventSender>,
    )>;
}

#[enum_dispatch(ClientTransportBuilder)]
pub(crate) enum ClientTransportBuilderEnum {
    #[cfg(not(target_family = "wasm"))]
    UdpSocket(UdpSocketBuilder),
    #[cfg(feature = "webtransport")]
    WebTransportClient(WebTransportClientSocketBuilder),
    #[cfg(feature = "websocket")]
    WebSocketClient(WebSocketClientSocketBuilder),
    LocalChannel(LocalChannelBuilder),
    Dummy(DummyIo),
}

#[allow(clippy::large_enum_variant)]
#[enum_dispatch(Transport)]
pub(crate) enum ClientTransportEnum {
    #[cfg(not(target_family = "wasm"))]
    UdpSocket(UdpSocket),
    #[cfg(feature = "webtransport")]
    WebTransportClient(WebTransportClientSocket),
    #[cfg(feature = "websocket")]
    WebSocketClient(WebSocketClientSocket),
    LocalChannel(LocalChannel),
    Dummy(DummyIo),
}

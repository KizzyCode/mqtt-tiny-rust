// Provides some type aliases that offer reasonable defaults for the underlying container types

// Sanity check
#[cfg(not(any(
    // Invalid use of `std` with `arrayvec` or `heapless`
    all(feature = "std", not(any(feature = "arrayvec", feature = "heapless"))),
    // Invalid use of `heapless` with `std` or `arrayvec`
    all(feature = "heapless", not(any(feature = "std", feature = "arrayvec"))),
    // Invalid use of `arrayvec` with `heapless` or `std`
    all(feature = "arrayvec", not(any(feature = "heapless", feature = "std"))),
)))]
// Raise a compiler error immediately
compile_error!("Must not use multiple backing features together (i.e. `std` or `arrayvec` or `heapless`)");

/// The default byte container type used within top-level types
#[cfg(feature = "std")]
#[doc(hidden)]
pub type Bytes = std::vec::Vec<u8>;
/// The default byte container type used within top-level types
///
/// # Note
/// This default configuration allows for 256 bytes per byte field on the stack.
#[cfg(feature = "arrayvec")]
#[doc(hidden)]
pub type Bytes = arrayvec::ArrayVec<u8, 256>;
/// The default byte container type used within top-level types
///
/// # Note
/// This default configuration allows for 256 bytes per byte field on the stack.
#[cfg(feature = "heapless")]
#[doc(hidden)]
pub type Bytes = heapless::Vec<u8, 256>;

/// The default collection type for topic lists used within top-level types
#[cfg(feature = "std")]
#[doc(hidden)]
pub type Topics = std::vec::Vec<Bytes>;
/// The default collection type for topic lists used within top-level types
///
/// # Note
/// This default configuration allows for 4 topics per unsubscribe message.
#[cfg(feature = "arrayvec")]
#[doc(hidden)]
pub type Topics = arrayvec::ArrayVec<Bytes, 4>;
/// The default collection type for topic lists used within top-level types
///
/// # Note
/// This default configuration allows for 4 topics per unsubscribe message.
#[cfg(feature = "heapless")]
#[doc(hidden)]
pub type Topics = heapless::Vec<Bytes, 4>;

/// The default collection type for topic+quality-of-service lists used within top-level types
#[cfg(feature = "std")]
#[doc(hidden)]
pub type TopicsQos = std::vec::Vec<(Bytes, u8)>;
/// The default collection type for topic+quality-of-service lists used within top-level types
///
/// # Note
/// This default configuration allows for 4 topic+quality-of-service tuples per subscribe message.
#[cfg(feature = "arrayvec")]
#[doc(hidden)]
pub type TopicsQos = arrayvec::ArrayVec<(Bytes, u8), 4>;
/// The default collection type for topic+quality-of-service lists used within top-level types
///
/// # Note
/// This default configuration allows for 4 topic+quality-of-service tuples per subscribe message.
#[cfg(feature = "heapless")]
#[doc(hidden)]
pub type TopicsQos = heapless::Vec<(Bytes, u8), 4>;

/// A type-erased MQTT packet
pub type Packet = crate::packets::packet::Packet<Topics, TopicsQos, Bytes>;
/// An MQTT [`CONNACK` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718033)
pub type Connack = crate::packets::connack::Connack;
/// An MQTT [`CONNECT` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718033)
pub type Connect = crate::packets::connect::Connect<Bytes>;
/// An MQTT [`DISCONNECT` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718090)
pub type Disconnect = crate::packets::disconnect::Disconnect;
/// An MQTT [`PINGREQ` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718081)
pub type Pingreq = crate::packets::pingreq::Pingreq;
/// An MQTT [`PINGRESP` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718086)
pub type Pingresp = crate::packets::pingresp::Pingresp;
/// An MQTT [`PUBACK` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718043)
pub type Puback = crate::packets::puback::Puback;
/// An MQTT [`PUBCOMP` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718058)
pub type Pubcomp = crate::packets::pubcomp::Pubcomp;
/// An MQTT [`PUBLISH` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718037)
pub type Publish = crate::packets::publish::Publish<Bytes>;
/// An MQTT [`PUBREC` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718048)
pub type Pubrec = crate::packets::pubrec::Pubrec;
/// An MQTT [`PUBREL` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718053)
pub type Pubrel = crate::packets::pubrel::Pubrel;
/// An MQTT [`SUBACK` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718068)
pub type Suback = crate::packets::suback::Suback;
/// An MQTT [`SUBSCRIBE` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718063)
pub type Subscribe = crate::packets::subscribe::Subscribe<TopicsQos, Bytes>;
/// An MQTT [`UNSUBACK` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718077)
pub type Unsuback = crate::packets::unsuback::Unsuback;
/// An MQTT [`UNSUBSCRIBE` packet](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718072)
pub type Unsubscribe = crate::packets::unsubscribe::Unsubscribe<Topics, Bytes>;

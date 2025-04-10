use std::fmt::Debug;
use downcast_rs::Downcast;
use serde::{Deserialize, Serialize};

macro_rules! zvariant_impl {
    ($ty:ty, $type_label:literal) => {
        #[typetag::serde(name = $type_label)]
        impl ZVariant for $ty {}
    };
    ($ty:ty) => {
        #[typetag::serde]
        impl ZVariant for $ty {}
    };
}


#[derive(Default, PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct ZString(String);

zvariant_impl!(ZString);
zvariant_impl!(TArray<ZString>, "TArray<ZString>");

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct ZEncryptedString;

zvariant_impl!(ZEncryptedString);
zvariant_impl!(TArray<ZEncryptedString>, "TArray<ZEncryptedString>");


#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct ZHMArenas;

zvariant_impl!(ZHMArenas);
zvariant_impl!(TArray<ZHMArenas>, "TArray<ZHMArenas>");


#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EntityTemplatePropertyId {
    Str(String),
    Num(u32),
}

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TArray<T: ZVariant>(Vec<T>);


#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "ZRepositoryID")]
pub struct ZRepositoryId;

zvariant_impl!(ZRepositoryId, "ZRepositoryID");
zvariant_impl!(TArray<ZRepositoryId>, "TArray<ZRepositoryID>");

#[typetag::serde(tag = "$type", content = "$val")]
pub trait ZVariant : Debug + Downcast
{}

zvariant_impl!(i8, "int8");
zvariant_impl!(TArray<i8>, "TArray<int8>");
zvariant_impl!(i16, "int16");
zvariant_impl!(TArray<i16>, "TArray<int16>");
zvariant_impl!(i32, "int32");
zvariant_impl!(TArray<i32>, "TArray<int32>");
zvariant_impl!(i64, "int64");
zvariant_impl!(TArray<i64>, "TArray<int64>");

zvariant_impl!(u8, "uint8");
zvariant_impl!(TArray<u8>, "TArray<uint8>");
zvariant_impl!(u16, "uint16");
zvariant_impl!(TArray<u16>, "TArray<uint16>");
zvariant_impl!(u32, "uint32");
zvariant_impl!(TArray<u32>, "TArray<uint32>");
zvariant_impl!(u64, "uint64");
zvariant_impl!(TArray<u64>, "TArray<uint64>");

zvariant_impl!(f32, "float32");
zvariant_impl!(TArray<f32>, "TArray<float32>");

zvariant_impl!(f64, "float64");
zvariant_impl!(TArray<f64>, "TArray<float64>");

zvariant_impl!(bool);
zvariant_impl!(TArray<bool>, "TArray<bool>");

zvariant_impl!(char);
zvariant_impl!(TArray<char>, "TArray<char>");

zvariant_impl!(String);
zvariant_impl!(TArray<String>, "TArray<String>");


#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "void")]
pub struct Empty;

zvariant_impl!(Empty, "void");

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "TypeID")]
pub struct TypeId;
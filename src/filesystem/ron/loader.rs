use ::serde::de::DeserializeSeed;
use bevy::asset::{AssetLoader, LoadedAsset};
use bevy::reflect::serde::UntypedReflectDeserializer;
use bevy_reflect::FromReflect;

use super::Ron;

#[derive(Default)]
pub struct RonLoader;

impl AssetLoader for RonLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            // let res: Ron = ron::de::from_bytes(bytes)?;

            let mut assets_path = std::env::current_exe()?;
            assets_path.pop();
            assets_path.push("assets");

            let assets_path = assets_path
                .to_str()
                .expect("RonLoader assets_path error: can't replace exe path")
                .replace("\\", "\\\\");

            let serialized = std::string::String::from_utf8(Vec::from(bytes))?
                .replace("__path_to_assets__", &assets_path);

            let mut registry = bevy::reflect::TypeRegistryInternal::new();
            registry.register::<Ron>();
            let reflect_deserializer = UntypedReflectDeserializer::new(&registry);
            let mut deserializer = ron::de::Deserializer::from_str(&serialized)?;
            let reflect_value = reflect_deserializer.deserialize(&mut deserializer)?;
            let ron = Ron::from_reflect(&*reflect_value).expect("RonLoader: Failed to deserialize ron");

            load_context.set_default_asset(LoadedAsset::new(ron));

            log::info!("RonLoader load ok, {}",  serialized);

            Ok(())
        })
    }
    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

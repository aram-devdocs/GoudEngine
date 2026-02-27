use goud_engine::ecs::Entity as EcsEntity;
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct Entity {
    pub(crate) inner: EcsEntity,
}

#[napi]
impl Entity {
    #[napi(constructor)]
    pub fn new(index: u32, generation: u32) -> Self {
        Self {
            inner: EcsEntity::new(index, generation),
        }
    }

    #[napi(factory)]
    pub fn placeholder() -> Self {
        Self {
            inner: EcsEntity::PLACEHOLDER,
        }
    }

    #[napi(factory)]
    pub fn from_bits(bits: BigInt) -> Result<Self> {
        let (_, value, _) = bits.get_u64();
        Ok(Self {
            inner: EcsEntity::from_bits(value),
        })
    }

    #[napi(getter)]
    pub fn index(&self) -> u32 {
        self.inner.index()
    }

    #[napi(getter)]
    pub fn generation(&self) -> u32 {
        self.inner.generation()
    }

    #[napi(getter)]
    pub fn is_placeholder(&self) -> bool {
        self.inner.is_placeholder()
    }

    #[napi]
    pub fn to_bits(&self) -> BigInt {
        BigInt::from(self.inner.to_bits())
    }

    #[napi]
    pub fn display(&self) -> String {
        format!("{}", self.inner)
    }
}

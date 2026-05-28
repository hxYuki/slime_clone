use std::{marker::PhantomData, sync::Arc};

use bevy::{
    asset::{Asset, AssetPath, AssetServer, Handle, meta::Settings},
    ecs::template::{FromTemplate, Template, TemplateContext},
};

pub struct AssetPathWithSettings<T: Asset, S: Settings, F: Fn(&mut S) + Send + Sync + 'static>(
    AssetPath<'static>,
    Arc<F>,
    PhantomData<(T, S)>,
);

impl<T: Asset, S: Settings, F: Fn(&mut S) + Send + Sync + 'static> Template
    for AssetPathWithSettings<T, S, F>
{
    type Output = Handle<T>;

    fn build_template(
        &self,
        context: &mut TemplateContext,
    ) -> bevy::ecs::error::Result<Self::Output> {
        let settings_fn = self.1.clone();
        Ok(context
            .resource::<AssetServer>()
            .load_builder()
            .with_settings(move |settings| (settings_fn)(settings))
            .load(self.0.clone()))
    }

    fn clone_template(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), std::marker::PhantomData)
    }
}

pub fn asset_with_settings<
    P: Into<AssetPath<'static>>,
    A: Asset,
    S: Settings,
    F: Fn(&mut S) + Send + Sync + 'static,
>(
    asset_path: P,
    settings_fn: F,
) -> AssetPathWithSettings<A, S, F> {
    AssetPathWithSettings(asset_path.into(), Arc::new(settings_fn), PhantomData)
}

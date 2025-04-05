use corrosive_asset_manager_macro::Asset;
use corrosive_ecs_core_macro::{Component, Resource};
use wgpu::RenderPipeline;
/*static mut PIPELINE_ASSETS: std::cell::LazyCell<
    corrosive_asset_manager::AssetManagerObject<PipelineAsset>,
> = std::cell::LazyCell::new(|| corrosive_asset_manager::AssetManagerObject::new());*/

#[derive(PartialEq, Resource, Asset)]
pub struct PipelineAsset {
    pub layout: RenderPipeline,
}

#[derive(PartialEq, Clone, Resource, Asset)]
pub struct ShaderAsset {
    pub shader: wgpu::ShaderModule,
}
#[derive(PartialEq, Resource, Asset)]
pub struct BindGroupLayoutAsset {
    pub layout: wgpu::BindGroupLayout,
}
/*impl corrosive_asset_manager::AssetObject for PipelineAsset {
    type AssetType = PipelineAsset;

    unsafe fn remove_asset(id: &u64) {
        PIPELINE_ASSETS
            .ref_counts
            .write()
            .expect(format!("Could not remove {} from PipelineAsset", id).as_str())
            .remove(id);
        PIPELINE_ASSETS
            .values
            .write()
            .expect(format!("Could not remove {} from PipelineAsset", id).as_str())
            .remove(id);
    }

    unsafe fn replace_asset(id: &u64, asset_object: Self::AssetType) {
        PIPELINE_ASSETS
            .values
            .write()
            .expect(format!("Could not join {} from PipelineAsset", id).as_str())
            .get_mut(id)
            .replace(&mut corrosive_asset_manager::AssetValue::Ready(
                asset_object,
            ));
    }

    unsafe fn add_asset<'a>(
        id: u64,
        asset_object: Self::AssetType,
    ) -> (
        &'a corrosive_asset_manager::AssetValue<'a, Self::AssetType>,
        &'a std::sync::atomic::AtomicUsize,
    ) {
        let ref_count: &std::sync::atomic::AtomicUsize = {
            let mut ref_count = PIPELINE_ASSETS
                .ref_counts
                .write()
                .expect(format!("Could not add {} to PipelineAsset", id).as_str());
            match ref_count.get_mut(&id) {
                None => std::mem::transmute(
                    ref_count
                        .entry(id)
                        .or_insert(std::sync::atomic::AtomicUsize::new(0)),
                ),
                Some(t) => {
                    t.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    std::mem::transmute(t)
                }
            }
        };
        let asset = std::mem::transmute(
            PIPELINE_ASSETS
                .values
                .write()
                .expect(format!("Could not add {} to PipelineAsset", id).as_str())
                .entry(id)
                .or_insert(corrosive_asset_manager::AssetValue::Ready(asset_object)),
        );
        (asset, ref_count)
    }
    unsafe fn load_asset<'a>(
        id: u64,
        asset_object: impl FnOnce() -> Self::AssetType + Send + 'static,
    ) -> (
        &'a corrosive_asset_manager::AssetValue<'a, Self::AssetType>,
        &'a std::sync::atomic::AtomicUsize,
    ) {
        let ref_count: &std::sync::atomic::AtomicUsize = {
            let mut ref_count = PIPELINE_ASSETS
                .ref_counts
                .write()
                .expect(format!("Could not add {} to PipelineAsset", id).as_str());
            match ref_count.get_mut(&id) {
                None => std::mem::transmute(
                    ref_count
                        .entry(id)
                        .or_insert(std::sync::atomic::AtomicUsize::new(0)),
                ),
                Some(t) => {
                    t.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    std::mem::transmute(t)
                }
            }
        };
        let binding = std::mem::transmute(&PIPELINE_ASSETS.default_value.get().read());
        let asset: &corrosive_asset_manager::AssetValue<PipelineAsset> = std::mem::transmute(
            PIPELINE_ASSETS
                .values
                .write()
                .expect(format!("Could not add {} to PipelineAsset", id).as_str())
                .entry(id)
                .or_insert(corrosive_asset_manager::AssetValue::NotReady(binding)),
        );
        std::thread::spawn(move || {
            PIPELINE_ASSETS
                .values
                .write()
                .expect(format!("Could not add {} to PipelineAsset", id).as_str())
                .insert(
                    id,
                    corrosive_asset_manager::AssetValue::Ready(asset_object()),
                )
        });
        (asset, ref_count)
    }

    unsafe fn set_default<'a>(asset_object: Self::AssetType)
    where
        <Self as corrosive_asset_manager::AssetObject>::AssetType:
            corrosive_asset_manager::AssetObject,
    {
        PIPELINE_ASSETS
            .default_value
            .get()
            .write(Some(asset_object));
    }
}*/

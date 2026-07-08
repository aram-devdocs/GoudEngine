//! Feature-gated local profiling helpers.

macro_rules! begin_frame {
    () => {
        #[cfg(all(not(feature = "profiling-tracy"), feature = "profiling-puffin"))]
        puffin::set_scopes_on(true);
    };
}

macro_rules! finish_frame {
    () => {
        #[cfg(feature = "profiling-tracy")]
        profiling::finish_frame!();
        #[cfg(all(not(feature = "profiling-tracy"), feature = "profiling-puffin"))]
        puffin::GlobalProfiler::lock().new_frame();
    };
}

macro_rules! profile_scope {
    (WGPU_BEGIN_FRAME $(,)?) => {
        $crate::libs::profiling::profile_scope!(@emit "wgpu.begin_frame");
    };
    (WGPU_END_FRAME $(,)?) => {
        $crate::libs::profiling::profile_scope!(@emit "wgpu.end_frame");
    };
    (WGPU_UNIFORM_UPLOAD $(,)?) => {
        $crate::libs::profiling::profile_scope!(@emit "wgpu.uniform_upload");
    };
    (WGPU_SHADOW_PASS $(,)?) => {
        $crate::libs::profiling::profile_scope!(@emit "wgpu.shadow_pass");
    };
    (WGPU_RENDER_PASS $(,)?) => {
        $crate::libs::profiling::profile_scope!(@emit "wgpu.render_pass");
    };
    (WGPU_GPU_SUBMIT $(,)?) => {
        $crate::libs::profiling::profile_scope!(@emit "wgpu.gpu_submit");
    };
    (ECS_RUN_SYSTEM $(,)?) => {
        $crate::libs::profiling::profile_scope!(@emit "ecs.run_system");
    };
    (ECS_SYSTEM_STAGE, $data:expr $(,)?) => {
        $crate::libs::profiling::profile_scope!(@emit "ecs.system_stage", $data);
    };
    (ECS_SYSTEM, $data:expr $(,)?) => {
        $crate::libs::profiling::profile_scope!(@emit "ecs.system", $data);
    };
    (ECS_PARALLEL_STAGE, $data:expr $(,)?) => {
        $crate::libs::profiling::profile_scope!(@emit "ecs.parallel_stage", $data);
    };
    (ECS_PARALLEL_SYSTEM, $data:expr $(,)?) => {
        $crate::libs::profiling::profile_scope!(@emit "ecs.parallel_system", $data);
    };
    (@emit $name:literal $(,)?) => {
        #[cfg(feature = "profiling-tracy")]
        profiling::scope!($name);
        #[cfg(all(not(feature = "profiling-tracy"), feature = "profiling-puffin"))]
        {
            puffin::set_scopes_on(true);
            puffin::profile_scope!($name);
        }
    };
    (@emit $name:literal, $data:expr $(,)?) => {
        #[cfg(feature = "profiling-tracy")]
        profiling::scope!($name, $data);
        #[cfg(all(not(feature = "profiling-tracy"), feature = "profiling-puffin"))]
        {
            puffin::set_scopes_on(true);
            puffin::profile_scope!($name, $data);
        }
    };
}

pub(crate) use begin_frame;
pub(crate) use finish_frame;
pub(crate) use profile_scope;

#[cfg(test)]
mod tests {
    #[cfg(feature = "profiling-tracy")]
    #[test]
    fn tracy_backend_macros_compile() {
        let _client = profiling::tracy_client::Client::start();

        profile_scope!(ECS_RUN_SYSTEM);
        finish_frame!();
    }

    #[cfg(all(not(feature = "profiling-tracy"), feature = "profiling-puffin"))]
    #[test]
    fn puffin_scope_enables_capture_without_wgpu_frame() {
        puffin::set_scopes_on(false);

        profile_scope!(ECS_RUN_SYSTEM);

        assert!(
            puffin::are_scopes_on(),
            "Puffin ECS scopes should not depend on entering the WGPU frame path first"
        );
        finish_frame!();
    }

    #[cfg(all(feature = "profiling-tracy", feature = "profiling-puffin"))]
    #[test]
    fn all_features_use_tracy_without_enabling_puffin() {
        puffin::set_scopes_on(false);
        let _client = profiling::tracy_client::Client::start();

        profile_scope!(ECS_RUN_SYSTEM);
        finish_frame!();

        assert!(
            !puffin::are_scopes_on(),
            "when both backends are enabled, the helper should follow the Tracy path"
        );
    }
}

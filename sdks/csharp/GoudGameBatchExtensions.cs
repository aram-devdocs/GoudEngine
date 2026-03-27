using System;
using System.Runtime.InteropServices;

namespace GoudEngine
{
    /// <summary>
    /// Batch operation wrappers for GoudGame that require manual pointer
    /// marshalling (unsafe fixed). These live outside the generated/ directory
    /// so codegen will not overwrite them.
    /// </summary>
    public partial class GoudGame
    {
        /// <summary>Creates multiple instances of a model in one call.</summary>
        public uint[] InstantiateModelBatch(uint sourceModelId, uint count)
        {
            var buf = new uint[count];
            int created = NativeMethods.goud_renderer3d_instantiate_model_batch(_ctx, sourceModelId, count, ref buf[0]);
            if (created < 0) return Array.Empty<uint>();
            var result = new uint[created];
            Array.Copy(buf, result, created);
            return result;
        }

        /// <summary>Sets positions for multiple models in one call.</summary>
        public unsafe int SetModelPositionsBatch(uint[] modelIds, float[] positions)
        {
            if (modelIds == null || positions == null) return -1;
            uint count = (uint)modelIds.Length;
            fixed (uint* idp = modelIds)
            fixed (float* posp = positions)
            {
                return NativeMethods.goud_renderer3d_set_model_positions_batch(_ctx, (IntPtr)idp, (IntPtr)posp, count);
            }
        }

        /// <summary>Adds multiple models to a scene in one call.</summary>
        public unsafe int AddModelsToSceneBatch(uint sceneId, uint[] modelIds)
        {
            if (modelIds == null) return -1;
            uint count = (uint)modelIds.Length;
            fixed (uint* p = modelIds)
            {
                return NativeMethods.goud_renderer3d_add_models_to_scene_batch(_ctx, sceneId, (IntPtr)p, count);
            }
        }
    }
}

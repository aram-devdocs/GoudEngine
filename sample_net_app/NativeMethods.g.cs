// <auto-generated>
// This code is generated by csbindgen.
// DON'T CHANGE THIS DIRECTLY.
// </auto-generated>
#pragma warning disable CS8500
#pragma warning disable CS8981
using System;
using System.Runtime.InteropServices;


namespace CsBindgen
{
    internal static unsafe partial class NativeMethods
    {
        const string __DllName = "libgame";



        [DllImport(__DllName, EntryPoint = "game_new", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern GameSdk* game_new(uint width, uint height, byte* title);

        [DllImport(__DllName, EntryPoint = "game_init", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern void game_init(GameSdk* game);

        [DllImport(__DllName, EntryPoint = "game_run", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern void game_run(GameSdk* game);

        [DllImport(__DllName, EntryPoint = "game_destroy", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern void game_destroy(GameSdk* game);


    }

    [StructLayout(LayoutKind.Sequential)]
    internal unsafe partial struct GameSdk
    {
    }



}

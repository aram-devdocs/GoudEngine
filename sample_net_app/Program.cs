﻿// // See https://aka.ms/new-console-template for more information
// Console.WriteLine("Hello, World!");


// get data from NativeMethods.g.cs namespace CsBindgen insternal static extern Game* game_new(uint width, uint height, byte* title);
// import from the neighbor file
using System;
using CsBindgen;




class Program
{
    static void Main(string[] args)
    {
        unsafe
        {
            fixed (byte* title = System.Text.Encoding.ASCII.GetBytes("Hello, World!"))
            {
                GameSdk* gamePtr = NativeMethods.game_new(800, 600, title);


                NativeMethods.game_init(gamePtr);

                NativeMethods.game_run(gamePtr);

                NativeMethods.game_destroy(gamePtr);

            }
        }
    }
}



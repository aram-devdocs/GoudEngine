using System;
using System.Runtime.InteropServices;

class RustInterop
{
    // Import the create_point function
    [DllImport("libsdk_bindings", CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr create_point(double x, double y);

    // Import the get_x function
    [DllImport("libsdk_bindings", CallingConvention = CallingConvention.Cdecl)]
    public static extern double get_x(IntPtr point);

    // Import the get_y function
    [DllImport("libsdk_bindings", CallingConvention = CallingConvention.Cdecl)]
    public static extern double get_y(IntPtr point);

    // Import the free_point function
    [DllImport("libsdk_bindings", CallingConvention = CallingConvention.Cdecl)]
    public static extern void free_point(IntPtr point);

    // Helper to get and display Point fields
    public static (double, double) GetPoint(IntPtr point)
    {
        double x = get_x(point);
        // Repeat for `y` (implement `get_y` in Rust if needed)
        // Assuming we added `get_y` for this example
        double y = get_y(point); // Implement get_y in Rust
        return (x, y);
    }

    static void Main()
    {
        // Create a new Point in Rust
        IntPtr point = create_point(3.0, 4.0);

        // Access Point fields
        (double x, double y) = GetPoint(point);
        Console.WriteLine($"Point: ({x}, {y})");

        // Free the Point memory
        free_point(point);
    }
}
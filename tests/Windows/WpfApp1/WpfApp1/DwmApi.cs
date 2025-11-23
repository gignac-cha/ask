using System;
using System.Runtime.InteropServices;

namespace WpfApp1
{
    /// <summary>
    /// P/Invoke wrapper for Desktop Window Manager (DWM) Thumbnail API.
    /// Provides functionality to create live, hardware-accelerated thumbnails of windows.
    /// </summary>
    public static class DwmApi
    {
        #region DWM Function Imports

        /// <summary>
        /// Creates a Desktop Window Manager (DWM) thumbnail relationship between the destination and source windows.
        /// </summary>
        /// <param name="hwndDestination">The handle to the window that will use the DWM thumbnail.</param>
        /// <param name="hwndSource">The handle to the window to use as the thumbnail source.</param>
        /// <param name="phThumbnailId">A pointer to a handle that will receive the DWM thumbnail handle.</param>
        /// <returns>If the function succeeds, it returns S_OK (0). Otherwise, it returns an error code.</returns>
        [DllImport("dwmapi.dll")]
        public static extern int DwmRegisterThumbnail(IntPtr hwndDestination, IntPtr hwndSource, out IntPtr phThumbnailId);

        /// <summary>
        /// Removes a Desktop Window Manager (DWM) thumbnail relationship.
        /// </summary>
        /// <param name="hThumbnailId">The DWM thumbnail handle to unregister.</param>
        /// <returns>If the function succeeds, it returns S_OK (0). Otherwise, it returns an error code.</returns>
        [DllImport("dwmapi.dll")]
        public static extern int DwmUnregisterThumbnail(IntPtr hThumbnailId);

        /// <summary>
        /// Updates the properties for a Desktop Window Manager (DWM) thumbnail.
        /// </summary>
        /// <param name="hThumbnailId">The DWM thumbnail handle.</param>
        /// <param name="ptnProperties">A pointer to a DWM_THUMBNAIL_PROPERTIES structure that contains the new thumbnail properties.</param>
        /// <returns>If the function succeeds, it returns S_OK (0). Otherwise, it returns an error code.</returns>
        [DllImport("dwmapi.dll")]
        public static extern int DwmUpdateThumbnailProperties(IntPtr hThumbnailId, ref DWM_THUMBNAIL_PROPERTIES ptnProperties);

        /// <summary>
        /// Retrieves the source size of the Desktop Window Manager (DWM) thumbnail.
        /// </summary>
        /// <param name="hThumbnailId">The DWM thumbnail handle.</param>
        /// <param name="pSize">A pointer to a SIZE structure that receives the size.</param>
        /// <returns>If the function succeeds, it returns S_OK (0). Otherwise, it returns an error code.</returns>
        [DllImport("dwmapi.dll")]
        public static extern int DwmQueryThumbnailSourceSize(IntPtr hThumbnailId, out PSIZE pSize);

        #endregion

        #region Structures

        /// <summary>
        /// Represents a size structure for DWM API.
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        public struct PSIZE
        {
            public int x;
            public int y;
        }

        /// <summary>
        /// Specifies Desktop Window Manager (DWM) thumbnail properties.
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        public struct DWM_THUMBNAIL_PROPERTIES
        {
            public int dwFlags;
            public RECT rcDestination;
            public RECT rcSource;
            public byte opacity;
            public bool fVisible;
            public bool fSourceClientAreaOnly;
        }

        /// <summary>
        /// Defines the coordinates of the upper-left and lower-right corners of a rectangle.
        /// </summary>
        [StructLayout(LayoutKind.Sequential)]
        public struct RECT
        {
            public int Left;
            public int Top;
            public int Right;
            public int Bottom;

            public RECT(int left, int top, int right, int bottom)
            {
                Left = left;
                Top = top;
                Right = right;
                Bottom = bottom;
            }

            public int Width => Right - Left;
            public int Height => Bottom - Top;
        }

        #endregion

        #region Constants

        /// <summary>
        /// A value for the dwFlags member that indicates the fVisible member contains valid information.
        /// </summary>
        public const int DWM_TNP_VISIBLE = 0x8;

        /// <summary>
        /// A value for the dwFlags member that indicates the opacity member contains valid information.
        /// </summary>
        public const int DWM_TNP_OPACITY = 0x4;

        /// <summary>
        /// A value for the dwFlags member that indicates the rcDestination member contains valid information.
        /// </summary>
        public const int DWM_TNP_RECTDESTINATION = 0x1;

        /// <summary>
        /// A value for the dwFlags member that indicates the rcSource member contains valid information.
        /// </summary>
        public const int DWM_TNP_RECTSOURCE = 0x2;

        /// <summary>
        /// A value for the dwFlags member that indicates the fSourceClientAreaOnly member contains valid information.
        /// </summary>
        public const int DWM_TNP_SOURCECLIENTAREAONLY = 0x10;

        #endregion
    }
}

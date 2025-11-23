using System;
using System.Diagnostics;
using System.Linq;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Input;
using System.Windows.Interop;
using OpenQA.Selenium;
using OpenQA.Selenium.Chrome;

namespace WpfApp1
{
    /// <summary>
    /// MainWindow.xaml에 대한 상호 작용 논리
    /// DWM Thumbnail API를 사용하여 실제 Chrome 브라우저를 PIP로 표시
    /// </summary>
    public partial class MainWindow : Window
    {
        private ChromeDriver chromeDriver;
        private IntPtr chromeWindowHandle = IntPtr.Zero;
        private IntPtr thumbnailHandle = IntPtr.Zero;
        private bool isPipActive = false;

        // Drag and resize state
        private bool isDragging = false;
        private bool isResizing = false;
        private Point dragStartPoint;

        public MainWindow()
        {
            InitializeComponent();
            this.Closed += MainWindow_Closed;
        }

        private async void LaunchPipButton_Click(object sender, RoutedEventArgs e)
        {
            if (!isPipActive)
            {
                await StartChromePip();
            }
            else
            {
                StopChromePip();
            }
        }

        private void NaverButton_Click(object sender, RoutedEventArgs e)
        {
            if (isPipActive && chromeDriver != null)
            {
                try
                {
                    chromeDriver.Navigate().GoToUrl("https://www.naver.com");
                    StatusText.Text = "Navigated to Naver.com";
                }
                catch (Exception ex)
                {
                    MessageBox.Show($"Navigation error: {ex.Message}", "Error", 
                        MessageBoxButton.OK, MessageBoxImage.Error);
                }
            }
            else
            {
                MessageBox.Show("Please launch Chrome PIP first!", "Info", 
                    MessageBoxButton.OK, MessageBoxImage.Information);
            }
        }

        private async Task StartChromePip()
        {
            try
            {
                StatusText.Text = "Launching Chrome browser...";

                // Step 1: ChromeDriver 옵션 설정
                var options = new ChromeOptions();
                options.AddArgument("--window-size=1920,1080");
                options.AddArgument("--window-position=-2000,0"); // 화면 밖에 배치
                options.AddArgument("--disable-infobars");
                options.AddArgument("--disable-extensions");
                options.AddArgument("--no-first-run");

                // Step 2: ChromeDriver 시작
                chromeDriver = new ChromeDriver(options);
                chromeDriver.Navigate().GoToUrl("https://www.google.com/search?q=ask+browser");

                StatusText.Text = "Finding Chrome window handle...";
                await Task.Delay(1500); // Chrome 창이 완전히 로드될 때까지 대기

                // Step 3: Chrome 프로세스의 메인 윈도우 핸들 찾기
                chromeWindowHandle = await FindChromeWindowHandle();

                if (chromeWindowHandle == IntPtr.Zero)
                {
                    MessageBox.Show("Failed to find Chrome window handle", "Error", 
                        MessageBoxButton.OK, MessageBoxImage.Error);
                    chromeDriver?.Quit();
                    return;
                }

                StatusText.Text = $"Chrome handle found: {chromeWindowHandle:X}";

                // Step 4: DWM Thumbnail 등록
                var thisWindowHandle = new WindowInteropHelper(this).Handle;
                int result = DwmApi.DwmRegisterThumbnail(thisWindowHandle, 
                    chromeWindowHandle, out thumbnailHandle);

                if (result != 0)
                {
                    MessageBox.Show($"Failed to register DWM thumbnail. Error: {result:X}", 
                        "Error", MessageBoxButton.OK, MessageBoxImage.Error);
                    chromeDriver?.Quit();
                    return;
                }

                // Step 5: Thumbnail 속성 설정 (위치, 크기, 투명도)
                UpdateThumbnailProperties();

                // Step 6: PIP 컨테이너 표시
                PipContainer.Visibility = Visibility.Visible;
                LaunchPipButton.Content = "Close Chrome PIP";
                isPipActive = true;

                StatusText.Text = "Chrome PIP is running (1920x1080 → 320x180)";
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Error starting Chrome PIP: {ex.Message}", 
                    "Error", MessageBoxButton.OK, MessageBoxImage.Error);
                StopChromePip();
            }
        }

        private void UpdateThumbnailProperties()
        {
            // PIP 컨테이너의 실제 화면 좌표 계산
            var pipPoint = PipContainer.TransformToAncestor(this).Transform(new Point(0, 0));
            int pipLeft = (int)pipPoint.X;
            int pipTop = (int)pipPoint.Y;
            int pipRight = pipLeft + (int)PipContainer.Width;
            int pipBottom = pipTop + (int)PipContainer.Height;

            var props = new DwmApi.DWM_THUMBNAIL_PROPERTIES
            {
                dwFlags = DwmApi.DWM_TNP_VISIBLE 
                        | DwmApi.DWM_TNP_RECTDESTINATION 
                        | DwmApi.DWM_TNP_OPACITY
                        | DwmApi.DWM_TNP_SOURCECLIENTAREAONLY,
                fVisible = true,
                opacity = 255, // 완전 불투명
                fSourceClientAreaOnly = true, // 브라우저 컨텐츠 영역만 (타이틀바 제외)
                rcDestination = new DwmApi.RECT(pipLeft, pipTop, pipRight, pipBottom),
                rcSource = new DwmApi.RECT(0, 0, 1920, 1080) // 소스 전체 영역
            };

            DwmApi.DwmUpdateThumbnailProperties(thumbnailHandle, ref props);
        }

        private async Task<IntPtr> FindChromeWindowHandle()
        {
            // 여러 번 시도하여 Chrome 프로세스 찾기
            for (int i = 0; i < 10; i++)
            {
                var chromeProcesses = Process.GetProcessesByName("chrome")
                    .Where(p => p.MainWindowHandle != IntPtr.Zero && 
                                !string.IsNullOrWhiteSpace(p.MainWindowTitle))
                    .ToList();

                // Google 검색 페이지가 로드된 창 찾기
                var targetProcess = chromeProcesses
                    .FirstOrDefault(p => p.MainWindowTitle.Contains("Google") || 
                                       p.MainWindowTitle.Contains("ask browser"));

                if (targetProcess != null)
                {
                    return targetProcess.MainWindowHandle;
                }

                // 혹은 가장 최근에 생성된 Chrome 창 사용
                if (chromeProcesses.Any())
                {
                    var latestChrome = chromeProcesses.OrderByDescending(p => p.StartTime).First();
                    if (latestChrome.MainWindowHandle != IntPtr.Zero)
                    {
                        return latestChrome.MainWindowHandle;
                    }
                }

                await Task.Delay(300); // 300ms 후 재시도
            }

            return IntPtr.Zero;
        }

        private void StopChromePip()
        {
            // DWM Thumbnail 해제
            if (thumbnailHandle != IntPtr.Zero)
            {
                DwmApi.DwmUnregisterThumbnail(thumbnailHandle);
                thumbnailHandle = IntPtr.Zero;
            }

            // ChromeDriver 종료
            try
            {
                chromeDriver?.Quit();
                chromeDriver?.Dispose();
                chromeDriver = null;
            }
            catch { }

            // UI 업데이트
            PipContainer.Visibility = Visibility.Collapsed;
            LaunchPipButton.Content = "Launch Chrome PIP";
            isPipActive = false;
            chromeWindowHandle = IntPtr.Zero;

            StatusText.Text = "Chrome PIP stopped";
        }

        private void MainWindow_Closed(object sender, EventArgs e)
        {
            StopChromePip();
        }

        // 윈도우 위치/크기 변경 시 Thumbnail 위치 업데이트
        protected override void OnLocationChanged(EventArgs e)
        {
            base.OnLocationChanged(e);
            if (isPipActive && thumbnailHandle != IntPtr.Zero)
            {
                UpdateThumbnailProperties();
            }
        }

        protected override void OnRenderSizeChanged(SizeChangedInfo sizeInfo)
        {
            base.OnRenderSizeChanged(sizeInfo);
            if (isPipActive && thumbnailHandle != IntPtr.Zero)
            {
                UpdateThumbnailProperties();
            }
        }

        #region Drag and Resize Event Handlers

        // PIP Container Drag
        private void PipContainer_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            if (isResizing) return; // Don't drag while resizing
            isDragging = true;
            dragStartPoint = e.GetPosition(this);
            PipContainer.CaptureMouse();
        }

        private void PipContainer_MouseMove(object sender, MouseEventArgs e)
        {
            if (isDragging && e.LeftButton == MouseButtonState.Pressed)
            {
                var currentPoint = e.GetPosition(this);
                var offset = currentPoint - dragStartPoint;

                PipContainer.Margin = new Thickness(
                    PipContainer.Margin.Left + offset.X,
                    PipContainer.Margin.Top + offset.Y,
                    0, 0);

                dragStartPoint = currentPoint;

                if (isPipActive && thumbnailHandle != IntPtr.Zero)
                {
                    UpdateThumbnailProperties();
                }
            }
        }

        private void PipContainer_MouseLeftButtonUp(object sender, MouseButtonEventArgs e)
        {
            if (isDragging)
            {
                isDragging = false;
                PipContainer.ReleaseMouseCapture();
            }
        }

        // Resize Grip
        private void ResizeGrip_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            isResizing = true;
            dragStartPoint = e.GetPosition(this);
            ResizeGrip.CaptureMouse();
            e.Handled = true; // Prevent container drag
        }

        private void ResizeGrip_MouseMove(object sender, MouseEventArgs e)
        {
            if (isResizing && e.LeftButton == MouseButtonState.Pressed)
            {
                var currentPoint = e.GetPosition(this);
                var offset = currentPoint - dragStartPoint;

                // Calculate new size maintaining aspect ratio (16:9)
                var newWidth = PipContainer.Width + offset.X;
                var newHeight = newWidth * 9.0 / 16.0; // Maintain 16:9 aspect ratio

                // Set minimum size
                if (newWidth >= 160 && newHeight >= 90)
                {
                    PipContainer.Width = newWidth;
                    PipContainer.Height = newHeight;

                    dragStartPoint = currentPoint;

                    if (isPipActive && thumbnailHandle != IntPtr.Zero)
                    {
                        UpdateThumbnailProperties();
                    }
                }
            }
        }

        private void ResizeGrip_MouseLeftButtonUp(object sender, MouseButtonEventArgs e)
        {
            if (isResizing)
            {
                isResizing = false;
                ResizeGrip.ReleaseMouseCapture();
                e.Handled = true;
            }
        }

        #endregion
    }
}

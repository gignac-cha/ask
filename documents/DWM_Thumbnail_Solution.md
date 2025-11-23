# Windows Browser PIP - DWM Thumbnail API 솔루션

## 개요

**DWM (Desktop Window Manager) Thumbnail API**를 사용하여 실제 Chrome 브라우저(1920x1080)를 WPF PIP 영역(320x180)에 실시간 라이브 스트리밍으로 보여주는 완벽한 솔루션입니다.

## 핵심 장점

1. **실시간 라이브 스트리밍**: GPU 가속으로 매우 낮은 CPU 사용량
2. **자동 스케일링**: Windows DWM이 자동으로 축소 렌더링
3. **ChromeDriver 완벽 호환**: Selenium으로 실제 브라우저 제어
4. **별도 라이브러리 불필요**: Windows 8+ 기본 제공

---

## 구현 방법

### 1단계: NuGet 패키지 설치

```xml
<!-- WpfApp1.csproj에 추가 -->
<PackageReference Include="Selenium.WebDriver">
  <Version>4.27.0</Version>
</PackageReference>
<PackageReference Include="Selenium.WebDriver.ChromeDriver">
  <Version>131.0.6778.8500</Version>
</PackageReference>
```

---

### 2단계: DWM API P/Invoke 선언

```csharp
using System;
using System.Runtime.InteropServices;

namespace WpfApp1
{
    public static class DwmApi
    {
        [DllImport("dwmapi.dll")]
        public static extern int DwmRegisterThumbnail(IntPtr dest, IntPtr src, out IntPtr thumb);

        [DllImport("dwmapi.dll")]
        public static extern int DwmUnregisterThumbnail(IntPtr thumb);

        [DllImport("dwmapi.dll")]
        public static extern int DwmUpdateThumbnailProperties(IntPtr hThumb, ref DWM_THUMBNAIL_PROPERTIES props);

        [DllImport("dwmapi.dll")]
        public static extern int DwmQueryThumbnailSourceSize(IntPtr thumb, out PSIZE size);

        [StructLayout(LayoutKind.Sequential)]
        public struct PSIZE
        {
            public int x;
            public int y;
        }

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
        }

        public const int DWM_TNP_VISIBLE = 0x8;
        public const int DWM_TNP_OPACITY = 0x4;
        public const int DWM_TNP_RECTDESTINATION = 0x1;
        public const int DWM_TNP_RECTSOURCE = 0x2;
        public const int DWM_TNP_SOURCECLIENTAREAONLY = 0x10;
    }
}
```

---

### 3단계: MainWindow.xaml 수정

```xml
<Window x:Class="WpfApp1.MainWindow"
        xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
        Title="Browser PIP Demo - DWM Thumbnail" Height="600" Width="800">
    <Grid>
        <!-- 중앙 버튼 -->
        <Button x:Name="LaunchPipButton"
                Content="Launch Chrome PIP"
                Width="200"
                Height="50"
                FontSize="16"
                HorizontalAlignment="Center"
                VerticalAlignment="Center"
                Click="LaunchPipButton_Click" />

        <!-- PIP 컨테이너: DWM Thumbnail이 렌더링될 영역 -->
        <Border x:Name="PipContainer"
                VerticalAlignment="Bottom"
                HorizontalAlignment="Left"
                Margin="20"
                BorderBrush="DodgerBlue"
                BorderThickness="3"
                Background="Black"
                Visibility="Collapsed"
                Width="320"
                Height="180">
            <!-- DWM이 직접 이 영역에 렌더링하므로 내부에 컨트롤 불필요 -->
            <Border.Effect>
                <DropShadowEffect BlurRadius="15" ShadowDepth="5" Opacity="0.5"/>
            </Border.Effect>
        </Border>

        <!-- 상태 표시 텍스트 (디버깅용) -->
        <TextBlock x:Name="StatusText"
                   VerticalAlignment="Top"
                   HorizontalAlignment="Left"
                   Margin="10"
                   FontSize="12"
                   Foreground="Gray"
                   Text="Click button to launch Chrome PIP" />
    </Grid>
</Window>
```

---

### 4단계: MainWindow.xaml.cs 완전 구현

```csharp
using System;
using System.Diagnostics;
using System.Linq;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Interop;
using OpenQA.Selenium;
using OpenQA.Selenium.Chrome;

namespace WpfApp1
{
    public partial class MainWindow : Window
    {
        private ChromeDriver chromeDriver;
        private IntPtr chromeWindowHandle = IntPtr.Zero;
        private IntPtr thumbnailHandle = IntPtr.Zero;
        private bool isPipActive = false;

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

                // Step 2: ChromeDriver 시작
                chromeDriver = new ChromeDriver(options);
                chromeDriver.Navigate().GoToUrl("https://www.google.com/?q=ask+browser");

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

                StatusText.Text = $"Chrome handle found: {chromeWindowHandle}";

                // Step 4: DWM Thumbnail 등록
                var thisWindowHandle = new WindowInteropHelper(this).Handle;
                int result = DwmApi.DwmRegisterThumbnail(thisWindowHandle, 
                    chromeWindowHandle, out thumbnailHandle);

                if (result != 0)
                {
                    MessageBox.Show($"Failed to register DWM thumbnail. Error: {result}", 
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
    }
}
```

---

## 작동 방식

### 프로세스 흐름

```
1. [ChromeDriver 시작]
   ↓
2. [Chrome 브라우저 실행 (1920x1080, 화면 밖)]
   ↓
3. [Google 검색 페이지 로드]
   ↓
4. [Chrome 윈도우 핸들 획득]
   ↓
5. [DWM Thumbnail 등록]
   ↓
6. [Thumbnail 속성 설정: 320x180으로 축소]
   ↓
7. [WPF PIP 영역에 실시간 렌더링] ← GPU 가속
```

### DWM의 역할

- **실시간 캡처**: Chrome 창의 내용을 지속적으로 모니터링
- **GPU 스케일링**: 1920x1080 → 320x180 자동 축소 (하드웨어 가속)
- **효율적 업데이트**: 변경된 부분만 업데이트
- **낮은 오버헤드**: CPU 사용량 거의 없음

---

## 비교: 다른 솔루션과의 차이

| 방식 | CPU 사용 | 실시간성 | 구현 복잡도 | 품질 |
|------|----------|----------|-------------|------|
| **DWM Thumbnail** | ⭐⭐⭐⭐⭐ 매우 낮음 | ⭐⭐⭐⭐⭐ 완벽 | ⭐⭐⭐ 중간 | ⭐⭐⭐⭐⭐ 최고 |
| WebView2 | ⭐⭐⭐⭐ 낮음 | ⭐⭐⭐⭐ 우수 | ⭐⭐⭐⭐⭐ 매우 쉬움 | ⭐⭐⭐⭐ 우수 |
| SetParent | ⭐⭐⭐ 보통 | ⭐⭐⭐⭐ 우수 | ⭐⭐ 어려움 | ⭐⭐⭐ 보통 |
| 화면 캡처 (30 FPS) | ⭐ 매우 높음 | ⭐⭐ 보통 | ⭐⭐⭐⭐ 쉬움 | ⭐⭐ 낮음 |
| DXGI Duplication | ⭐⭐ 높음 | ⭐⭐⭐⭐⭐ 완벽 | ⭐ 매우 어려움 | ⭐⭐⭐⭐⭐ 최고 |

---

## 시스템 요구사항

- **OS**: Windows 8 이상 (DWM Thumbnail API 필요)
- **.NET Framework**: 4.7.2 이상
- **Chrome/ChromeDriver**: 최신 버전
- **Edge WebView2 Runtime**: 불필요 (실제 Chrome 사용)

---

## 주의사항

1. **핸들 찾기 타이밍**: Chrome이 완전히 로드될 때까지 1~2초 대기 필요
2. **여러 Chrome 창**: 타이틀로 구분하거나 가장 최근 창 사용
3. **리소스 해제**: 앱 종료 시 반드시 `DwmUnregisterThumbnail` 호출
4. **윈도우 이동**: WPF 창이 이동/리사이즈되면 `UpdateThumbnailProperties` 재호출 필요

---

## 추가 개선 사항

### 1. PIP 위치 조정 가능하게 만들기

```csharp
// PipContainer를 드래그 가능하게
private void PipContainer_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
{
    if (e.ClickCount == 2)
    {
        // 더블클릭으로 크기 토글
        if (PipContainer.Width == 320)
        {
            PipContainer.Width = 640;
            PipContainer.Height = 360;
        }
        else
        {
            PipContainer.Width = 320;
            PipContainer.Height = 180;
        }
        UpdateThumbnailProperties();
    }
}
```

### 2. 투명도 조절

```csharp
props.opacity = 200; // 0~255 (200 = 약간 투명)
```

### 3. 브라우저 컨텐츠만 표시

```csharp
props.fSourceClientAreaOnly = true; // 타이틀바/툴바 제외
```

---

## 결론

**DWM Thumbnail API**는 실제 Chrome 브라우저를 PIP로 보여주는 **최고의 솔루션**입니다:

✅ **실제 1920x1080 브라우저** 실행  
✅ **PIP 영역에 320x180으로 축소** 표시  
✅ **라이브 스트리밍** (실시간 업데이트)  
✅ **GPU 가속**으로 낮은 CPU 사용  
✅ **ChromeDriver**로 완전한 제어  

이 방식은 WebView2보다 실제 Chrome을 사용하므로 확장 프로그램, 개발자 도구 등 모든 기능을 활용할 수 있습니다!

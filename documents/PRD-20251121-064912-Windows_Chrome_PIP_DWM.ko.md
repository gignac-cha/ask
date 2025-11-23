# 제품 요구 사항 문서 (PRD): Windows Chrome PIP (DWM Thumbnail)

**날짜:** 2025-11-21
**프로젝트명:** Windows Chrome PIP (DWM 에디션)
**버전:** 2.0
**상태:** 초안

---

## A. 정보 수집 및 검증 (웹 검색)

실제 1920x1080 Chrome 브라우저 인스턴스를 축소된 PIP 창으로 표시하라는 요구 사항을 달성하기 위해 광범위한 기술 조사가 수행되었습니다.

*   **솔루션 비교**:
    *   **WebView2**: 거부됨. 효율적이지만, 특정 자동화/확장 프로그램 요구 사항을 위해 *실제* Chrome 브라우저 인스턴스를 사용하지 않음.
    *   **SetParent (창 임베딩)**: 거부됨. Chrome과 같은 복잡한 다중 프로세스 애플리케이션을 `SetParent`를 통해 WPF 창에 임베딩하면 심각한 안정성, 렌더링("Airspace"), 입력 포커스 문제가 발생함.
    *   **화면 캡처 (BitBlt/DXGI)**: 거부됨. 지속적인 스트리밍 시 높은 CPU 사용량과 진정한 실시간 "라이브" 느낌 부족.
    *   **DWM Thumbnail API**: **선택됨**. Windows DWM(Desktop Window Manager)은 한 창의 실시간 하드웨어 가속 반사(reflection)를 다른 창에 렌더링하는 네이티브 API(`DwmRegisterThumbnail`)를 제공함. 이를 통해 전체 1920x1080 Chrome 창을 CPU 오버헤드가 거의 없고 완벽한 시각적 충실도로 320x180으로 축소하여 표시할 수 있음.

*   **기술적 타당성**:
    *   Windows 8 이상 필요 (대상: Windows 11).
    *   `dwmapi.dll` P/Invoke 호출 필요.
    *   "소스" Chrome 창을 제어하기 위해 Selenium WebDriver 필요.

---

## B. 서브 에이전트 전략 (병렬 실행)

이 복잡한 통합을 효율적으로 실행하기 위해 다음과 같은 AI 페르소나가 정의됩니다.

*   **서브 에이전트 [Win32 상호 운용성 전문가]**:
    *   **책임**: 관리되지 않는(unmanaged) `dwmapi.dll` 함수 호출 정의 및 구현.
    *   **작업**: `DwmRegisterThumbnail`, `DwmUpdateThumbnailProperties` 및 관련 구조체(`RECT`, `DWM_THUMBNAIL_PROPERTIES`)를 포함하는 `DwmApi` 정적 클래스 생성. 올바른 P/Invoke 서명 보장.
*   **서브 에이전트 [브라우저 자동화 엔지니어]**:
    *   **책임**: Chrome 브라우저 수명 주기 관리.
    *   **작업**: Selenium ChromeDriver 로직을 구현하여 Chrome을 화면 밖(또는 숨김 상태)에서 실행하고, 대상 URL로 이동하며, 브라우저 프로세스의 올바른 창 핸들(HWND)을 안정적으로 검색.
*   **서브 에이전트 [WPF UI 개발자]**:
    *   **책임**: 호스팅 UI 생성.
    *   **작업**: XAML에서 PIP 컨테이너 설계. DWM은 창 표면에 *직접* 렌더링하므로, UI 컨테이너는 주로 DWM 렌더링 사각형의 자리 표시자 및 좌표 참조 역할을 함.

---

## C. 단계별 구현 계획

### 1단계: 인프라 및 종속성
**목표**: Win32 API 호출 및 Selenium 자동화를 위한 프로젝트 준비.
**산출물**:
*   NuGet 패키지 설치: `Selenium.WebDriver`, `Selenium.WebDriver.ChromeDriver`.
*   필요한 모든 P/Invoke 정의 및 상수를 캡슐화하는 `DwmApi.cs` 생성.
*   프로젝트가 호환되는 .NET 버전(4.7.2+ 또는 .NET 6+)을 대상으로 하는지 확인.

### 2단계: Chrome 자동화 코어
**목표**: 실제 Chrome 인스턴스를 성공적으로 실행하고 제어.
**산출물**:
*   `ChromeManager` 클래스 구현.
*   특정 인수로 Chrome 실행 (`--window-size=1920,1080`, 메인 뷰에서 숨기려면 `--window-position`을 화면 밖으로 설정).
*   실행된 Chrome 인스턴스의 특정 `MainWindowHandle`을 찾는 견고한 로직 구현 (다중 프로세스 아키텍처 처리).

### 3단계: DWM 통합 ("PIP" 로직)
**목표**: Chrome 창을 WPF 애플리케이션에 연결.
**산출물**:
*   `RegisterThumbnail` 로직 구현: Chrome HWND(소스)를 WPF Window HWND(대상)에 연결하는 `DwmRegisterThumbnail` 호출.
*   `UpdateThumbnail` 로직 구현: WPF PIP 컨테이너의 화면 좌표를 계산하여 `DwmUpdateThumbnailProperties`에 전달.
*   DWM 속성 구성: `Opacity`, `Visible`, `SourceClientAreaOnly`(선호하는 경우 Chrome의 제목 표시줄 제거) 설정.

### 4단계: 상호 작용 및 정리
**목표**: 사용자 경험을 다듬고 리소스 안전 보장.
**산출물**:
*   **토글 로직**: PIP 세션을 시작/중지하는 버튼.
*   **창 관리**: WPF 창이 이동하거나 크기가 조정될 때 썸네일이 업데이트되도록 보장 (`LocationChanged` 이벤트 처리).
*   **정리**: 애플리케이션 종료 시 좀비 프로세스나 메모리 누수를 방지하기 위해 `DwmUnregisterThumbnail` 및 `ChromeDriver.Quit()`의 중요한 구현.

---

## D. 기술 사양

*   **소스 해상도**: 1920 x 1080 (논리적).
*   **대상 해상도**: ~320 x 180 (시각적 PIP).
*   **스케일링 방식**: DWM 하드웨어 스케일링 (GPU에서 제공하는 Bilinear/Anisotropic).
*   **입력 처리**: DWM 썸네일은 *시각적 전용*임. 기본적으로 클릭을 허용하지 않음. ("PIP 뷰"에 기반한 디스플레이 전용 요구 사항).
*   **창 핸들 전략**: 재시도 로직이 있는 `Process.MainWindowHandle` 또는 네이티브 HWND에 매핑된 Selenium의 `CurrentWindowHandle` 사용.

---
*PRD 끝*

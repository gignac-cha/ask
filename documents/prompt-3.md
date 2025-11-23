# Role
당신은 **수석 프로덕트 매니저(Chief Product Manager)**이자 **시스템 아키텍트**입니다. 
당신의 목표는 아래 `---` 구분선 밑에 제공될 사용자의 요구사항을 분석하여, 개발 팀이 즉시 실행 가능한 수준의 완벽한 **PRD(제품 요구사항 정의서)**를 작성하는 것입니다.

# Output Specification (필수 준수 사항)
작성되는 PRD는 반드시 다음의 제약 사항과 형식을 따라야 합니다.

## 1. 파일 생성 규칙
- 생성된 PRD는 반드시 다음 경로와 포맷으로 저장/출력한다고 명시해야 합니다:
  `documents/PRD-{YYYYMMDD-HHmmss}-{English_Summary_Of_Project}.md`
  *(예: documents/PRD-20231027-143000-TodoList_App.md)*

## 2. 콘텐츠 제약 사항 (엄격)
- **[금지]** PRD 내부에 구체적인 **예제 소스 코드(Source Code)**를 절대 작성하지 마십시오. (구현 로직은 텍스트와 다이어그램으로만 설명합니다.)
- **[금지]** PRD 내부에 구체적인 **폴더 구조(File Tree)**를 절대 명시하지 마십시오.

## 3. 필수 포함 내용 및 전략
작성되는 PRD는 다음의 섹션과 전략을 반드시 포함해야 합니다.

### A. 정보 수집 및 검증 (Web Search)
- 사용자의 요구사항만으로 기술적 세부 사항(최신 라이브러리 버전, API 스펙, 경쟁 서비스 벤치마킹 등)이 부족하다고 판단될 경우, **반드시 웹 검색을 통해 정보를 수집한 후** 그 내용을 바탕으로 PRD를 작성한다고 명시하십시오.
- 어떤 정보를 검색하여 반영했는지 요약하여 기재하십시오.

### B. 병렬 작업 구조화 (Sub-agents Strategy)
- 프로젝트 내에서 서로 의존성 없이 독립적으로 진행 가능한 작업(예: 프론트엔드 UI 디자인, 백엔드 스키마 설계, 테스팅 시나리오 작성 등)을 식별하십시오.
- 각 독립 작업에 대해 **최적화된 AI 페르소나(Sub-agent)**를 정의하고, 최대한 병렬적으로 작업을 수행하도록 지시하는 내용을 포함하십시오.
  - *예시 포맷: "Sub-agent [UI Designer]: 메인 대시보드 와이어프레임 설계"*

### C. 단계별 실행 계획 (Phased Implementation)
- 전체 프로젝트를 논리적이고 순차적인 **Phase(단계)**로 나누어 작성하십시오.
- 작업의 예상 규모와 복잡도에 따라 Phase의 수를 유동적으로 조절하십시오. (단순한 경우 2~3단계, 복잡한 경우 5단계 이상 등)
- 각 Phase는 명확한 목표(Goal)와 산출물(Deliverables)을 포함해야 합니다.

---
# User Requirements (여기서부터 사용자의 요구사항입니다)

(여기에 프로젝트에 대한 구체적인 아이디어, 기능, 목표 등을 자유롭게 작성하세요.)

tests/Windows/WpfApp1 앱과 같은 컨셉의 앱을 macOS 에서도 사용할 수 있게 tests/MacOS 폴더에 swift 로 macOS 내이티브 앱을 개발해줘

DWM (Desktop Window Manager) Thumbnail API는 Windows 운영체제 전용 기술입니다. 따라서 이 PRD에 작성된 기술(dwmapi.dll, DwmRegisterThumbnail)을 그대로 맥(macOS)에서 사용할 수는 없습니다.
하지만 맥에서도 **다른 기술(API)**을 사용하여 **동일한 기능(브라우저 창을 PIP로 띄우기)**을 구현할 수는 있습니다.


# macOS Browser PIP Tech Stack

## 1. Core Frameworks (Native)
*   **Language**: Swift 5.x+
*   **UI Framework**: SwiftUI (권장) 또는 AppKit (NSWindow, NSView)
    *   *이유*: 최신 macOS 앱 개발 표준이며, Floating Window(PIP) 구현이 용이함.

## 2. Window Capture & Rendering (The "DWM" Alternative)
Windows의 `DWM Thumbnail`을 대체하는 macOS의 핵심 기술입니다.

*   **ScreenCaptureKit (SCK)** (macOS 12.3+)
    *   *역할*: 고성능 화면/창 캡처.
    *   *기능*: `SCShareableContent`를 통해 현재 실행 중인 모든 창 목록(Chrome 포함)을 가져오고, 특정 `WindowID`를 타겟팅하여 비디오 스트림(`CMSampleBuffer`)을 생성합니다.
    *   *장점*: CPU 사용량이 매우 낮고 GPU 가속을 지원합니다. (기존 `CGWindowListCreateImage`의 현대적 대체재)

*   **Core Graphics (CG)**
    *   *역할*: 창 식별 (Legacy/Fallback).
    *   *기능*: `CGWindowListCopyWindowInfo`를 사용하여 Chrome 브라우저의 Process ID(PID)와 Window ID를 초기에 검색할 때 보조적으로 사용될 수 있습니다.

*   **Metal (MTKView) 또는 AVFoundation (AVSampleBufferDisplayLayer)**
    *   *역할*: 캡처된 스트림 렌더링.
    *   *기능*: ScreenCaptureKit에서 넘어오는 비디오 프레임을 내 앱의 PIP 윈도우에 실시간으로 그립니다.

## 3. Browser Automation (Control)
브라우저를 실행하고 특정 URL로 이동시키는 제어 기술입니다.

*   **Selenium WebDriver (for macOS)**
    *   *역할*: Chrome 브라우저 실행 및 제어.
    *   *구성*: `Selenium.WebDriver` + `ChromeDriver` (macOS용 바이너리).
    *   *기능*: Windows 버전과 동일한 로직(URL 이동, 창 크기 설정 등)을 공유할 수 있습니다.

*   **AppleScript (NSAppleScript)** (Optional)
    *   *역할*: 간단한 제어 보조.
    *   *기능*: Selenium 없이 이미 열려있는 Chrome 탭을 제어하거나 창을 포커싱할 때 사용할 수 있는 macOS 네이티브 스크립팅 도구입니다.

## 4. Implementation Architecture
1.  **Browser Launch**: Selenium으로 Chrome을 실행 (`--window-size=1920,1080`).
2.  **Target Discovery**: `SCShareableContent`를 쿼리하여 Selenium이 실행한 Chrome의 `WindowID`를 찾음.
3.  **Stream Setup**: `SCStream`을 생성하여 해당 `WindowID`만 캡처하도록 필터링.
4.  **Rendering**: 캡처된 프레임을 내 앱의 `Floating NSWindow` 내부 `View`에 그림.

기존에 작업한 PRD 목록
* PRD-20251121-061955-WPF_Browser_PIP_Window.md
* PRD-20251121-062617-Windows_Browser_PIP.md
* PRD-20251121-064912-Windows_Chrome_PIP_DWM.md
# 제품 요구사항 정의서 (PRD): Windows Browser PIP

**날짜:** 2025-11-21
**프로젝트명:** Windows Browser PIP
**버전:** 1.0
**상태:** 초안 (Draft)

---

## A. 정보 수집 및 검증 (Web Search)

요구사항 정의에 앞서 웹 검색을 통해 기술적 타당성을 검증했습니다.

*   **Visual Studio 2026 컨텍스트**: Visual Studio 2026이 최신 안정 버전(2025년 11월 GA)이며 .NET 10을 지원함을 확인했습니다. 최신 WebView2 성능 개선을 활용하기 위해 .NET 10(또는 사용자 환경이 제한적인 경우 최신 지원 버전)을 타겟으로 합니다.
*   **WebView2 스케일링**:
    *   **난제**: 1920x1080 논리적 브라우저 화면을 약 320x240 크기의 시각적 컨테이너 안에 표시해야 함.
    *   **해결책**: 일반적인 WPF `Viewbox`는 WebView2와 함께 사용 시 렌더링 아티팩트가 발생할 수 있습니다. 권장되는 방식은 WebView2 컨트롤의 `RenderTransform`에 `ScaleTransform`을 적용하는 것입니다.
    *   **계산**: 1920 너비를 320 너비에 맞추기 위해 약 `0.166`의 스케일 팩터가 필요합니다.
    *   **종횡비 참고**: 1920x1080은 16:9 비율이고, 320x240은 4:3 비율입니다. 16:9를 4:3에 꽉 채우면 레터박스(상하 여백)가 생기거나 잘릴 수 있습니다. 본 구현에서는 너비(320px)를 맞추는 것을 우선하며, 이 경우 높이는 약 180px가 되어 240px 컨테이너 내 중앙에 배치되거나 컨테이너 높이를 180px로 조정합니다.

---

## B. 병렬 작업 구조화 (Sub-agents Strategy)

효율성을 극대화하기 위해 다음과 같이 AI 페르소나(Sub-agents)를 정의하여 병렬 작업을 수행합니다:

*   **Sub-agent [UI Architect]**:
    *   **책임**: `MainWindow`의 XAML 레이아웃 설계.
    *   **작업**: Grid 레이아웃 정의, 중앙 "Open PIP" 버튼 배치, 좌측 하단 PIP 뷰를 위한 `Border` 컨테이너 정의. WebView2 플레이스홀더에 `RenderTransform` 속성 적용.
*   **Sub-agent [Core Logic Dev]**:
    *   **책임**: C# 코드 비하인드 및 WebView2 초기화 구현.
    *   **작업**: 버튼 클릭 이벤트 처리, WebView2 초기화(High DPI 필요 시 인자 전달), 타겟 URL 내비게이션, PIP 창의 가시성(Visibility) 토글 관리.
*   **Sub-agent [QA Specialist]**:
    *   **책임**: 해상도 및 상호작용 테스트 케이스 정의.
    *   **작업**: 브라우저가 화면상 물리적으로는 ~320x240 픽셀을 차지하지만, 내부적으로는 1920x1080 뷰포트로 인식되는지 확인(JS 콘솔 체크).

---

## C. 단계별 실행 계획 (Phased Implementation)

### Phase 1: 프로젝트 초기화 및 구성
**목표**: 필요한 종속성이 포함된 안정적인 빌드 환경 구축.
**산출물**:
*   `tests/Windows` 폴더에 솔루션 생성.
*   NuGet 패키지 `Microsoft.Web.WebView2` 설치.
*   `app.manifest`를 Per-Monitor DPI 인식으로 설정 (올바른 WebView2 렌더링을 위해 필수).

### Phase 2: UI 레이아웃 및 PIP 컨테이너 설정
**목표**: 로직 없이 시각적 구조 생성.
**산출물**:
*   **메인 윈도우**: 표준 WPF 윈도우 (800x600 또는 기본값).
*   **중앙 버튼**: "Launch Browser PIP" 라벨, Grid 중앙 배치.
*   **PIP 컨테이너**: 좌측 하단에 고정된 `Grid` 또는 `Border` (VerticalAlignment="Bottom", HorizontalAlignment="Left").
*   **WebView2 컨트롤**: PIP 컨테이너 내부에 배치.
    *   *중요 설정*: XAML에서 WebView2 컨트롤 크기를 명시적으로 **1920x1080**으로 지정 (Width="1920", Height="1080").
    *   *스케일링*: `ScaleTransform`을 `0.166` (X/Y)으로 적용하여 시각적으로 ~320px 너비로 축소.
    *   *기준점*: `RenderTransformOrigin`을 "0,1" (좌측 하단)로 설정하거나 마진을 조정하여 시각적 컨테이너 내에 올바르게 정렬.

### Phase 3: 상호작용 로직 및 내비게이션
**목표**: 애플리케이션 기능 구현.
**산출물**:
*   **버튼 이벤트**: 클릭 시 PIP 컨테이너의 가시성을 `Collapsed`에서 `Visible`로 토글.
*   **내비게이션**: `WebView2.CoreWebView2.Navigate("https://www.google.com/?q=ask+browser")` 호출.
*   **수명 주기 관리**: 내비게이션 전 WebView2가 비동기적으로 초기화되도록 보장.

### Phase 4: 검증 및 개선
**목표**: 요구사항 충족 여부 확인 및 UX 부드러움 확보.
**산출물**:
*   **해상도 확인**: JavaScript `alert(window.innerWidth + 'x' + window.innerHeight)`를 주입하여 사이트가 1920x1080으로 인식하는지 확인.
*   **시각적 확인**: 텍스트가 작게(축소되어) 보이지만 레이아웃이 모바일 뷰가 아닌 전체 데스크톱 브라우저와 일치하는지 확인.
*   **종횡비 처리**: 16:9 콘텐츠가 4:3 공간 내에서 중앙 정렬되거나 의도대로 정렬되는지 확인.

---

## D. 기술 사양 및 제약 사항

*   **타겟 프레임워크**: .NET 10 (Visual Studio 2026 경유) 또는 .NET Framework 4.8 (레거시 필요한 경우). *권장: 더 나은 WebView2 지원을 위해 .NET 6+ 사용.*
*   **라이브러리**: `Microsoft.Web.WebView2` (최신 안정 버전).
*   **해상도 로직**:
    *   **논리적 크기**: 1920px x 1080px.
    *   **시각적 크기**: ~320px x 180px (종횡비 유지).
    *   **구현 상세**:
        ```text
        <WebView2 Width="1920" Height="1080">
            <WebView2.RenderTransform>
                <ScaleTransform ScaleX="0.1666" ScaleY="0.1666" />
            </WebView2.RenderTransform>
        </WebView2>
        ```
        *참고: 스케일된 높이가 원하는 영역을 초과할 경우 컨테이너가 경계를 클리핑해야 하지만, 180px는 240px 내에 충분히 들어갑니다.*

---
*PRD 종료*

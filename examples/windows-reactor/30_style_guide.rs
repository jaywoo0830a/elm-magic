//! windows-reactor 예제 30 — **스타일 가이드(종합)**: 토큰·색·타이포·간격·버튼·상태
//! **업스트림 대응**: `gallery` — 업스트림의 "여러 주제를 한 앱으로" 샘플과 같은 역할이다.
//! 다른 점은 이 저장소가 **규칙을 테스트로 고정**한다는 것(업스트림 `gallery`는 시연 위주).
//!
//!
//! **언제 쓰나**: 예제 21~29를 **하나의 앱**으로 묶을 때. 이 예제가 그 결론이다 —
//! 토큰(21) · 타이포(22) · 간격(23) · 모양(24) · 색(25) · 버튼(26) · 레이아웃(27) ·
//! 스크롤(28) · 상태(29)가 한 화면에서 **같은 규칙**으로 동작한다.
//!
//! **구성(누가 무엇을 소유하는가)**
//! - **호스트**: 창 테마(`WindowVisuals::theme`)와 선택된 섹션. → props + 콜백 prop으로 내린다
//!   (예제 11의 계약 그대로).
//! - **elm**: 사이드바(섹션 버튼) · 테마 토글 · 브레드크럼. 콜백 prop을 부를 뿐이다.
//! - **`<Raw>`**: 본문 한 덩어리(스크롤 영역 + 섹션 내용). 렌더 본문에서 `guide(dark)`와
//!   `section_body(index, guide)`를 계산해 클로저가 캡처한다(예제 21의 방법).
//!
//! **이 앱이 지키는 규칙(테스트가 고정한다)**
//! 1. 토큰은 [`Guide`] 하나에서만 나온다 — 화면에 매직 넘버가 없다.
//! 2. 기하 토큰(radius/gap)은 테마와 무관하고, 색 토큰만 테마를 따른다.
//! 3. 표면은 **테마 브러시**로 만든다 — 라이트/다크/고대비가 공짜로 따라온다.
//! 4. 브랜드 색은 테마마다 **직접** 준다(Reactor에 색 계산 API가 없다 — 예제 25).
//! 5. 섹션 목록([`SECTIONS`])과 화면 라벨이 어긋나면 테스트가 잡는다.
//!
//! **주의**
//! - `<Raw>` 안에서는 **elm 상태를 읽을 수 없다**(본문이 그대로 복사되기 때문) — 그래서 섹션
//!   내용은 전부 값(=`Guide`, `index`)으로 넘긴다. 이 제약이 이 어댑터의 유일한 "문법"이다.
//! - `<Raw>`의 클로저는 `Fn`이라 캡처한 값을 **이동할 수 없다** — 필요하면 `clone()`한다.
//!
//! **함정: `view!` 본문의 최상위 요소는 하나여야 한다**
//! - `view!` 본문은 `into_element({ ... })` **블록**으로 감싸진다. 최상위 요소가 여럿이면
//!   **마지막 하나만 값이 되고 앞의 것들은 버려진다**(문장이 되어 값이 사라진다 — 조용히).
//!   그래서 이 예제는 본문 전체를 `<Col>` 하나로 감쌌다(`<Row>` + 브레드크럼).
//! - 컨테이너(`<Col>`/`<Row>`)의 자식은 개수 제한이 없다 — 안쪽은 마음껏 나눠도 된다.
//! - 재현: `ui! { <Text>"first"</Text> <Text>"second"</Text> }`의 텍스트는 `"second"`뿐이다.

use elm_magic::prelude::*;
use elm_magic_windows_reactor::{ElmInput, ElmView, RawSlot};
use windows_reactor::{
    App, Border, Brush, Button, ButtonStyle, ChildrenControl, Color, Component, ComponentContext,
    ContentControl, CornerRadius, FontWeight, InfoBar, InfoBarSeverity, KeyedView, LayoutControl,
    Orientation, ProgressBar, ProgressRing, ResourceOverrides, ScrollBarVisibility, ScrollViewer,
    StackPanel, TextBlock, ThemeBrush, Thickness, VariableSizedWrapGrid, View, ViewContext,
    WindowBackdrop, WindowTheme, WindowVisuals,
};

/// 이 가이드가 쓰는 토큰 — **모든 화면이 여기서만 값을 가져온다**.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Guide {
    dark: bool,
    /// 모서리 반지름 — 기하 토큰(테마와 무관).
    radius: f64,
    /// 간격 단위 — 기하 토큰(테마와 무관).
    gap: f64,
    /// 글자 크기 — 타이포 토큰.
    title: f64,
    body: f64,
    /// 표면/테두리 — **테마 브러시**(값이 아니라 이름이다).
    surface: ThemeBrush,
    stroke: ThemeBrush,
    /// 브랜드 색 — 테마마다 직접 준다.
    brand: Color,
}

/// 테마 → 토큰. 기하는 그대로, 색만 바뀐다.
fn guide(dark: bool) -> Guide {
    Guide {
        dark,
        radius: 6.0,
        gap: 8.0,
        title: 20.0,
        body: 13.0,
        surface: ThemeBrush::CardBackground,
        stroke: ThemeBrush::CardStroke,
        brand: if dark {
            Color::rgb(108, 176, 255)
        } else {
            Color::rgb(0, 95, 184)
        },
    }
}

/// 섹션 목록 — 표와 화면이 같은 출처를 본다.
///
/// 사이드바 라벨은 `view!` 본문이 리터럴로 쓰고, 이 표는 `#[cfg(test)]`가
/// 화면 라벨과 대조한다 — `allow`는 "테스트 전용 계약"이라는 표시다.
#[allow(dead_code)]
const SECTIONS: [(&str, &str); 6] = [
    ("토큰", "radius · gap · 글자 크기"),
    ("색", "테마 브러시 8종 + 브랜드"),
    ("타이포", "크기 단계 + 굵기"),
    ("간격", "단위의 배수만"),
    ("버튼", "변형 4종"),
    ("상태", "정상 / 로딩 / 오류 / 비활성"),
];

/// 섹션 내용을 세로로 쌓는 공통 틀 — 모든 섹션이 같은 간격을 쓴다.
fn column(guide: Guide, children: Vec<View>) -> View {
    StackPanel::new().spacing(guide.gap).keyed_children(
        children
            .into_iter()
            .enumerate()
            .map(|(index, view)| KeyedView::new(index as u64, view)),
    )
}

/// 이 가이드의 표면 — **한 곳에서만** 만든다(모서리/테두리/여백이 화면마다 달라지지 않게).
fn surface(guide: Guide, text: &str) -> View {
    Border::new()
        .background(Brush::from(guide.surface))
        .border_brush(Brush::from(guide.stroke))
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(CornerRadius::uniform(guide.radius))
        .padding(Thickness::uniform(guide.gap))
        .content(TextBlock::new().text(text).font_size(guide.body))
}

/// WCAG 상대 휘도 — 배경 위에 얹을 글자색을 **계산으로** 고른다(예제 25의 규칙).
fn luminance(color: Color) -> f64 {
    let channel = |value: u8| {
        let value = f64::from(value) / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
}

fn readable_on(background: Color) -> Color {
    if luminance(background) > 0.5 {
        Color::rgb(0, 0, 0)
    } else {
        Color::rgb(255, 255, 255)
    }
}

/// 0) 토큰 — 이 가이드가 쓰는 값 자체를 보여준다.
fn token_section(guide: Guide) -> View {
    column(
        guide,
        vec![
            TextBlock::new()
                .text(format!(
                    "radius {} · gap {} · title {} · body {}",
                    guide.radius, guide.gap, guide.title, guide.body
                ))
                .font_size(guide.body)
                .into(),
            TextBlock::new()
                .text(if guide.dark {
                    "테마: dark — 색 토큰만 바뀐다"
                } else {
                    "테마: light — 색 토큰만 바뀐다"
                })
                .font_size(guide.body - 1.0)
                .opacity(0.6)
                .into(),
            surface(guide, "표면 = CardBackground + CardStroke + radius"),
        ],
    )
}

/// 1) 색 — 테마 브러시 8종과 브랜드 색.
fn color_section(guide: Guide) -> View {
    const THEME_BRUSHES: [(ThemeBrush, &str); 8] = [
        (ThemeBrush::Accent, "Accent"),
        (ThemeBrush::AccentText, "AccentText"),
        (ThemeBrush::PrimaryText, "PrimaryText"),
        (ThemeBrush::SolidBackground, "SolidBackground"),
        (ThemeBrush::CardBackground, "CardBackground"),
        (ThemeBrush::CardStroke, "CardStroke"),
        (ThemeBrush::SystemCritical, "SystemCritical"),
        (
            ThemeBrush::SystemCriticalBackground,
            "SystemCriticalBackground",
        ),
    ];

    let swatches: Vec<KeyedView> = THEME_BRUSHES
        .iter()
        .enumerate()
        .map(|(index, (brush, name))| {
            KeyedView::new(
                index as u64,
                StackPanel::new().spacing(4.0).children((
                    Border::new()
                        .width(96.0)
                        .height(26.0)
                        .background(Brush::from(*brush))
                        .border_brush(Brush::from(guide.stroke))
                        .border_thickness(Thickness::uniform(1.0))
                        .corner_radius(CornerRadius::uniform(4.0)),
                    TextBlock::new().text(*name).font_size(10.0).opacity(0.6),
                )),
            )
        })
        .collect();

    column(
        guide,
        vec![
            VariableSizedWrapGrid::new()
                .item_width(110.0)
                .item_height(52.0)
                .orientation(Orientation::Horizontal)
                .keyed_children(swatches),
            Border::new()
                .width(120.0)
                .height(32.0)
                .background(guide.brand)
                .corner_radius(CornerRadius::uniform(guide.radius))
                .padding(Thickness::uniform(6.0))
                .content(
                    TextBlock::new()
                        .text("brand")
                        .font_size(guide.body - 2.0)
                        .foreground(readable_on(guide.brand)),
                ),
        ],
    )
}

/// 2) 타이포 — 크기 단계와 굵기. 크기는 토큰에서만 나온다.
fn type_section(guide: Guide) -> View {
    column(
        guide,
        vec![
            TextBlock::new()
                .text("Aa 디자인")
                .font_size(guide.title)
                .font_weight(FontWeight::BOLD)
                .into(),
            TextBlock::new()
                .text("Aa 섹션 제목")
                .font_size(guide.title - 4.0)
                .font_weight(FontWeight::SEMI_BOLD)
                .into(),
            TextBlock::new()
                .text("Aa 본문")
                .font_size(guide.body)
                .into(),
            TextBlock::new()
                .text("Aa 보조 설명 — 색을 더 만들지 않고 opacity로 낮춘다")
                .font_size(guide.body - 1.0)
                .opacity(0.6)
                .into(),
            surface(
                guide,
                "폰트 패밀리/행간 API는 없다 — 크기·굵기·색·자름이 전부다",
            ),
        ],
    )
}

/// 3) 간격 — 단위의 배수만 쓴다(예제 23).
fn spacing_section(guide: Guide) -> View {
    let bars: Vec<KeyedView> = [1.0, 2.0, 3.0, 4.0, 6.0]
        .iter()
        .enumerate()
        .map(|(index, multiple)| {
            let width = guide.gap * multiple;
            KeyedView::new(
                index as u64,
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(guide.gap / 2.0)
                    .children((
                        Border::new()
                            .width(width)
                            .height(10.0)
                            .background(Brush::from(ThemeBrush::Accent))
                            .corner_radius(CornerRadius::uniform(2.0)),
                        TextBlock::new()
                            .text(format!("gap ×{multiple} = {width}"))
                            .font_size(10.0)
                            .opacity(0.6),
                    )),
            )
        })
        .collect();

    column(
        guide,
        vec![
            StackPanel::new()
                .spacing(guide.gap / 2.0)
                .keyed_children(bars),
            surface(guide, "간격은 단위의 배수만 — 리듬이 곧 정돈이다"),
        ],
    )
}

/// 4) 버튼 — 변형 4종과 브랜드 오버라이드(예제 26).
fn button_section(guide: Guide) -> View {
    let branded = Button::new()
        .resource_overrides(
            ResourceOverrides::new()
                .set("ButtonBackground", guide.brand)
                .set("ButtonForeground", readable_on(guide.brand))
                .set("ButtonCornerRadius", CornerRadius::uniform(guide.radius)),
        )
        .content("브랜드 버튼");

    column(
        guide,
        vec![
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(guide.gap)
                .children((
                    Button::new().style(ButtonStyle::Accent).content("Primary"),
                    Button::new()
                        .style(ButtonStyle::Default)
                        .content("Secondary"),
                    Button::new().style(ButtonStyle::Subtle).content("Ghost"),
                    Button::new().is_enabled(false).content("Disabled"),
                )),
            branded,
        ],
    )
}

/// 5) 상태 — 색이 아니라 **위젯**으로 표현한다(예제 29).
fn state_section(guide: Guide) -> View {
    column(
        guide,
        vec![
            ProgressRing::new().is_active(true).into(),
            ProgressBar::new().value(60.0).show_error(true).into(),
            InfoBar::new()
                .severity(InfoBarSeverity::Warning)
                .title("주의")
                .message("상태는 색이 아니라 위젯 · 투명도 · is_enabled로 표현한다")
                .is_open(true)
                .into(),
            surface(
                guide,
                "비활성 표현은 WinUI가 만든다 — 우리는 is_enabled만 준다",
            ),
        ],
    )
}

/// 섹션 인덱스 → 내용. 모르는 값은 첫 섹션으로 본다.
fn section_body(index: i32, guide: Guide) -> View {
    match index {
        1 => color_section(guide),
        2 => type_section(guide),
        3 => spacing_section(guide),
        4 => button_section(guide),
        5 => state_section(guide),
        _ => token_section(guide),
    }
}

/// 본문은 스크롤 영역에 넣는다(예제 28).
fn scrolled(content: View) -> View {
    ScrollViewer::new()
        .vertical_scroll_bar_visibility(ScrollBarVisibility::Auto)
        .content(content)
}

elm_magic::view! {
    fn StyleGuide(section = 0, dark = true, on_section: fn(i32), on_theme: fn(bool)) {
        // `<Raw>`가 캡처할 값은 렌더 본문에서 계산한다(예제 21).
        let guide = guide(dark);
        let index = section;

        // 본문 전체를 **하나의 최상위 `<Col>`**로 감싼다 — view! 본문은 블록이라
        // 최상위 요소가 여럿이면 마지막 하나만 값이 된다(문서 헤더의 함정 참고).
        <Col>
            <Row>
                <Col>
                    <Button on_click={on_section(0)}>"토큰"</Button>
                    <Button on_click={on_section(1)}>"색"</Button>
                    <Button on_click={on_section(2)}>"타이포"</Button>
                    <Button on_click={on_section(3)}>"간격"</Button>
                    <Button on_click={on_section(4)}>"버튼"</Button>
                    <Button on_click={on_section(5)}>"상태"</Button>

                    <If when={dark}>
                        <Button on_click={on_theme(false)}>"라이트로"</Button>
                    <Else>
                        <Button on_click={on_theme(true)}>"다크로"</Button>
                    </Else>
                    </If>
                </Col>

                <Raw>|out: &mut RawSlot| {
                    *out = Some(scrolled(section_body(index, guide)));
                }</Raw>
            </Row>

            <Switch on={section}>
                <Case when={0}><Text>"섹션: 토큰"</Text></Case>
                <Case when={1}><Text>"섹션: 색"</Text></Case>
                <Case when={2}><Text>"섹션: 타이포"</Text></Case>
                <Case when={3}><Text>"섹션: 간격"</Text></Case>
                <Case when={4}><Text>"섹션: 버튼"</Text></Case>
                <Default><Text>"섹션: 상태"</Text></Default>
            </Switch>
        </Col>
    }
}

/// 호스트 메시지 — elm의 콜백 prop이 큐에 넣는다(예제 11).
#[derive(Clone)]
enum HostMessage {
    Section(i32),
    Theme(bool),
}

/// 호스트: 창 테마와 선택된 섹션을 소유한다.
struct Host {
    section: i32,
    dark: bool,
}

impl Component for Host {
    type Input = ();
    type Message = HostMessage;

    fn create(_input: &(), _context: &ComponentContext<Self>) -> Self {
        Self {
            section: 0,
            dark: true,
        }
    }

    fn update(&mut self, message: HostMessage, _context: &ComponentContext<Self>) {
        match message {
            HostMessage::Section(section) => self.section = section,
            HostMessage::Theme(dark) => self.dark = dark,
        }
    }

    fn view(&self, _input: &(), context: &mut ViewContext<Self>) -> View {
        context.window_title("elm-magic — 스타일 가이드");
        // 창 자체의 테마/배경은 호스트만 바꿀 수 있다(예제 21).
        context.window_visuals(
            WindowVisuals::new()
                .theme(if self.dark {
                    WindowTheme::Dark
                } else {
                    WindowTheme::Light
                })
                .backdrop(WindowBackdrop::Mica),
        );

        let section_sender = context.sender();
        let on_section = Callback::new(move |_arena, section: i32| {
            let _ = section_sender.send(HostMessage::Section(section));
        });
        let theme_sender = context.sender();
        let on_theme = Callback::new(move |_arena, dark: bool| {
            let _ = theme_sender.send(HostMessage::Theme(dark));
        });

        View::component::<ElmView<StyleGuide>>(ElmInput::new(StyleGuideProps {
            section: Some(self.section),
            dark: Some(self.dark),
            on_section: Some(on_section),
            on_theme: Some(on_theme),
            ..Default::default()
        }))
    }
}

fn main() {
    App::run_component::<Host>(()).expect("Reactor 실행 실패");
}

#[cfg(test)]
mod tests {
    use super::*;
    use elm_magic_windows_reactor::{plan, Pass, PlanNode};

    fn plan_for(section: i32, dark: bool) -> (PlanNode, Pass) {
        let mut ctx = Ctx::new();
        let tree = elm_magic::frame::<StyleGuide>(
            &mut ctx,
            &StyleGuideProps {
                section: Some(section),
                dark: Some(dark),
                ..Default::default()
            },
        );
        plan(&tree)
    }

    /// 섹션 표는 6개이고, 제목/설명이 모두 채워져 있다.
    #[test]
    fn the_section_table_is_complete() {
        assert_eq!(SECTIONS.len(), 6);
        for (title, note) in SECTIONS {
            assert!(!title.is_empty(), "제목이 비었다");
            assert!(!note.is_empty(), "{title}의 설명이 비었다");
        }
    }

    /// 기하 토큰은 테마와 무관하고, 색 토큰만 테마를 따른다.
    #[test]
    fn only_color_tokens_follow_the_theme() {
        let light = guide(false);
        let dark = guide(true);
        assert_eq!(light.radius, dark.radius);
        assert_eq!(light.gap, dark.gap);
        assert_eq!(light.title, dark.title);
        assert_eq!(light.body, dark.body);
        assert_eq!(
            light.surface, dark.surface,
            "테마 브러시는 값이 아니라 이름이다"
        );
        assert_ne!(light.brand, dark.brand, "브랜드는 테마마다 직접 준다");
    }

    /// 브랜드 색 위의 글자는 어느 테마에서도 읽힌다(예제 25의 대비 규칙).
    #[test]
    fn the_brand_color_is_readable_in_both_themes() {
        for dark in [false, true] {
            let brand = guide(dark).brand;
            assert_ne!(readable_on(brand), brand, "글자와 배경이 같은 색일 수 없다");
        }
        assert_eq!(readable_on(guide(false).brand), Color::rgb(255, 255, 255));
    }

    /// 섹션 목록과 화면 라벨이 일치한다 — 표와 화면이 어긋나면 여기서 잡힌다.
    #[test]
    fn the_section_labels_match_the_table() {
        for (index, (title, _)) in SECTIONS.iter().enumerate() {
            let (_, pass) = plan_for(index as i32, true);
            assert!(
                pass.has_text(&format!("섹션: {title}")),
                "{title} 섹션의 라벨이 없다"
            );
        }
    }

    /// 본문은 하나의 Raw(스크롤 영역)이고, 사이드바는 elm이 그린다.
    #[test]
    fn the_plan_has_one_raw_block() {
        let (node, pass) = plan_for(0, true);
        assert_eq!(pass.count("Raw"), 1, "본문만 Raw다");
        assert_eq!(pass.count("Button"), 7, "섹션 6 + 테마 1");
        assert_eq!(pass.count("StackPanel"), 3, "바깥 Col + Row + 사이드바 Col");
        let kinds: Vec<&str> = node.children.iter().map(|c| c.control()).collect();
        assert_eq!(
            kinds,
            vec!["StackPanel", "TextBlock"],
            "Row 하나와 브레드크럼 하나"
        );
    }

    /// 사이드바 클릭이 호스트 콜백으로 간다(예제 11의 계약).
    #[test]
    fn the_sidebar_reports_the_section_to_the_host() {
        use std::cell::Cell;
        use std::rc::Rc;

        let picked = Rc::new(Cell::new(-1));
        let seen = Rc::clone(&picked);
        let props = StyleGuideProps {
            section: Some(0),
            dark: Some(true),
            on_section: Some(Callback::new(move |_arena, section: i32| {
                seen.set(section);
            })),
            ..Default::default()
        };

        let mut app = elm_magic::mount_with::<StyleGuide>(props);
        app.click("색");
        assert_eq!(picked.get(), 1, "elm이 호스트 콜백을 불러야 한다");
    }
}

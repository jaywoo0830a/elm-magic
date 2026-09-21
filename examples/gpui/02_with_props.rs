//! gpui 예제 02 — props 지정 (`ElmView::with_props`)
//!
//! **언제 쓰나**: 화면을 부모 상태에서 분리해 재사용할 때. egui의
//! `frame::<C>(ctx, &CProps { .. })`에 대응하는 gpui 쪽 API가 `with_props`다.
//!
//! **핵심**
//! - `ElmView::<C>::new(cx)` — `C::Props: Default`일 때만. 내부적으로
//!   `with_props(C::Props::default(), cx)`를 부른다.
//! - `ElmView::<C>::with_props(props, cx)` — 명시적 props.
//! - props는 **엔티티를 만들 때 복사**된다. props가 바뀌면 엔티티를 새로 만들어
//!   교체한다(`cx.new(..)` 다시 호출) — 엔티티 안의 `props` 필드를 밖에서 바꾸는
//!   공개 API는 없다.
//! - 필수 prop(기본값 없는 매개변수)을 `None`으로 두면 첫 렌더에서 panic한다 —
//!   `with_props`에서 채워야 한다.
//!
//! **베스트 패턴**: props를 **부모 엔티티가 소유**하고, 부모가 다시 렌더될 때 자식
//! 엔티티를 만들거나 캐시한다. 화면 파라미터(선택된 탭, 문서 id)처럼 잘 안 바뀌는
//! 값만 props로 넘기고, 자주 바뀌는 값은 `#[store]`(예제 14)로 둔다.

use elm_magic::prelude::*;
use elm_magic_gpui::ElmView;
use gpui_kit::{div, AppContext as _, Context, IntoElement, ParentElement as _, Render, Window};

elm_magic::view! {
    // `title`은 필수 prop, `count`는 기본값 0.
    fn Panel(title: String, count = 0) {
        <Col>
            "{title}"
            "count: {count}"
            <Button on_click={count += 1}>"inc"</Button>
        </Col>
    }
}

struct Root {
    /// 부모가 화면 파라미터를 소유한다.
    title: String,
}

impl Render for Root {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let props = PanelProps {
            title: Some(self.title.clone()),
            count: None, // 기본값
            ..Default::default() // children 포함
        };
        div().child(cx.new(|cx| ElmView::<Panel>::with_props(props, cx)))
    }
}

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        let root = Root { title: "패널".to_string() };
        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| root))
                .expect("failed to open window");
        })
        .detach();
    });
}
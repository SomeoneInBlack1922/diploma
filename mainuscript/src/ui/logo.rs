use relm4::{gtk::prelude::{BoxExt, WidgetExt}, prelude::*};
use gtk::{Box, Label};
pub struct LogoView;
impl SimpleComponent for LogoView{
    type Init = ();
    type Output = ();
    type Root = Box;
    type Input = ();
    type Widgets = ();
    fn init_root() -> Self::Root {
        let root = gtk::Box::builder()
            .width_request(300)
            .orientation(gtk::Orientation::Horizontal)
            .height_request(15)
            .build();
        root.add_css_class("logo_view");
        root
    }
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self>
    {
        let m_letter = Label::new(Some("m"));
        let ai_part = Label::new(Some("AI"));
        let nuscript_part = Label::new(Some("nuscript"));

        m_letter.add_css_class("logo-base");
        ai_part.add_css_class("logo-contrast");
        nuscript_part.add_css_class("logo-base");

        //Assemble root
        root.append(&m_letter);
        root.append(&ai_part);
        root.append(&nuscript_part);
        ComponentParts { model: LogoView {}, widgets: () }
    }
}
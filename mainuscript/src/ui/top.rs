use std::sync::Arc;

use relm4::SimpleComponent;
use gtk::{Window, Box as GtkBox};
use relm4::prelude::*;
use relm4::gtk;
use gtk::prelude::*;
use crate::ui::navigation::NavigationView;
use crate::ui::navigation::NavigationEvent;
use crate::ui::work_area::settings::SettingsView;
use crate::bus::Bus;

pub struct TopWidgets {
    window: Window
}
pub struct TopView{
    bus: Arc<Bus>,
    top_container: GtkBox,
    current_working_area: GtkBox
}
#[derive(Debug)]
pub enum TopInput{
    Navigation(NavigationEvent)
}
impl SimpleComponent for TopView{
    type Input = TopInput;
    type Output = ();
    type Init = Bus;
    type Root = Window;
    type Widgets = ();
    fn init_root() -> Self::Root {
        Window::builder()
            .title("mainuscript")
            .height_request(800)
            .width_request(1500)
            .decorated(true)
            .vexpand(true)
            .maximized(true)
            .build()
    }
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self>
    {
        let bus = Arc::new(init);
        //UI
        let top_container = GtkBox::new(gtk::Orientation::Horizontal, 0);
        let working_area_empty = GtkBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            .build();
        working_area_empty.add_css_class("working_area_empty");
        working_area_empty.append(&gtk::Label::new(Some("EMPTY WORKING AREA")));

        let navigation_bar_connector = NavigationView::builder()
            // .attach_to(&top_container)
            .launch(bus.clone());
        let navigation_bar_controller = navigation_bar_connector.forward(sender.input_sender(), |navigation_output|{
            return TopInput::Navigation(navigation_output)
        });
        top_container.append(navigation_bar_controller.widget());
        top_container.append(&working_area_empty);

        root.set_child(Some(&top_container));

        ComponentParts {
            model: TopView {
                bus,
                top_container: top_container,
                current_working_area: working_area_empty
            },
            widgets: ()
        }
    }
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message{
            TopInput::Navigation(navigation_event) => {
                match navigation_event{
                    // Need to open settings in working area
                    NavigationEvent::Settings => {
                        tracing::info!("requested to open Settings");
                        // Remove previous working area
                        self.top_container.remove(&self.current_working_area);
                        // Construct settings element
                        let setting_builder = SettingsView::builder();
                        let settings_root = &setting_builder.root;
                        
                        // Add settings wiew to the container
                        self.top_container.append(settings_root);
                        // Store settings as current working area
                        self.current_working_area = settings_root.clone();
                        setting_builder.launch(());
                    },
                    NavigationEvent::Object(_) => {
                        todo!()
                    }
                }
            }
        }
    }
    
}
fn new_top_container() -> GtkBox{
    GtkBox::new(gtk::Orientation::Horizontal, 0)
}
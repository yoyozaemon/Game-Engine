#![allow(non_snake_case)]

use bitflags::bitflags;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum EventType
{
    None = 0,
	WindowResized, WindowClosed, WindowMoved,
	KeyPressed, KeyReleased, KeyTyped,
	MouseButtonPressed, MouseButtonReleased, MouseScrolled, MouseMoved
}

bitflags!
{
    pub struct EventCategory: u8
    {
        const NONE         = 0b00000000;
        const KEYBOARD     = 0b00000001;
        const APPLICATION  = 0b00000010;
        const MOUSE        = 0b00000100;
        const INPUT        = 0b00001000;
        const MOUSE_BUTTON = 0b00010000;
    }
}

pub trait Event
{
    fn GetType(&self) -> EventType;
    fn GetName(&self) -> &str;
    fn GetCategoryFlags(&self) -> EventCategory;
    fn IsInCategory(&self, category: EventCategory) -> bool { return (self.GetCategoryFlags() & category) != EventCategory::NONE; }
    fn AsAny(&self) -> &dyn std::any::Any;
    fn ToString(&self) -> String;
}
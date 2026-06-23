use std::sync::Arc;

use winit::event_loop::EventLoop;

use crate::{cartridge::Cartridge, nes::NES, renderer::Renderer};

pub mod bit_utils;

pub mod memory_bus;

pub mod cpu;
pub mod instructions;

pub mod ppu;
pub mod apu;
pub mod controller;

pub mod mapper;
pub mod mappers;

pub mod cartridge;

pub mod nes;

pub mod renderer;


struct AppState<'a> {
    nes:        NES<'a>,
    renderer:   Renderer
}

struct App<'a> {
    state: Option<AppState<'a>>
}

impl<'a> winit::application::ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_some() { return; }

        let cart = Cartridge::load("tests/roms/Donkey Kong.nes").unwrap();

        let mut state = AppState { 
            nes: NES::new(cart, None, None),
            renderer: Renderer::new(Arc::new(event_loop.create_window(Default::default()).unwrap()))
        };

        state.nes.reset();

        state.renderer.window.set_min_inner_size(Some(winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(256, 240))));
        
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) 
    {
        let Some(state) = &mut self.state else {
            return;
        };

        if state.renderer.window.id() != window_id { return; }

        match event {
            winit::event::WindowEvent::Resized(size) => {
                state.renderer.config.width = size.width;
                state.renderer.config.height = size.height;

                state.renderer.surface.configure(&state.renderer.device, &state.renderer.config);
            }

            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            winit::event::WindowEvent::RedrawRequested => {
                let f = state.nes.bus.ppu.frame;

                while state.nes.bus.ppu.frame == f {
                    state.nes.tick();
                }

                state.renderer.upload_framebuffer(&state.nes.framebuffer);
                state.renderer.render();

                state.renderer.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.run_app(&mut App{ state: None }).unwrap();
}
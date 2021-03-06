// Ecore Rust bindings for EFL.
// Copyright (C) 2014  Luis Araujo <araujoc.luisf@gmail.com>

// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version.

// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.

// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA


extern crate libc;

use std::ptr;
use std::option::Option;
use std::mem::transmute;

use ecore::libc::{c_int, c_char, c_void};
use eseful::{to_c_args, EventInfo};
use eo;
use eina;
use evas;


pub enum EcoreEvas {}

pub enum EcoreEventHandler {}

pub enum EcoreEvent {
    EcoreEventNone,
    EcoreEventSignalUser,
    EcoreEventSignalHup,
    EcoreEventSignalExit,
    EcoreEventSignalPower,
    EcoreEventSignalRealtime,
    EcoreEventMemoryState,
    EcoreEventPowerState,
    EcoreEventLocaleChanged,
    EcoreEventHostnameChanged,
    EcoreEventSystemTimedateChanged,
    EcoreEventCount
}

pub struct SigInfo;

pub struct EcoreEventSignalExit {
    pub interrupt: eina::EinaBool,
    pub quit: eina::EinaBool,
    pub terminate: eina::EinaBool,
    pub ext_data: *c_void,
    pub data: SigInfo
}

type EcoreTimer = eo::Eo;

pub static ECORE_CALLBACK_RENEW: eina::EinaBool = eina::EINA_TRUE;

/* High level callback notation */
pub type EcoreTaskCb<T> = fn (&Option<T>) -> eina::EinaBool;
/* C level callback notation */
type CEcoreTaskCb = fn (*c_void) -> u8;

/* High level callback notation */
pub type EcoreEventHandlerCb<T> = fn (&Option<T>, int, &EventInfo) -> bool;
/* C level callback notation */
type CEcoreEventHandlerCb = fn (*c_void, c_int, *c_void) -> u8;

pub type EcoreEvasEventCb = fn (&EcoreEvas);
type _CEcoreEvasEventCb = fn (*EcoreEvas);

#[link(name = "ecore")]
extern "C" {
    fn ecore_init() -> c_int;
    fn ecore_app_args_set(argc: c_int, argv: **c_char);
    fn ecore_main_loop_begin();
    fn ecore_main_loop_quit();
    fn ecore_time_get() -> f64;
    fn ecore_time_unix_get() -> f64;
    fn ecore_shutdown() -> c_int;
    fn ecore_timer_add(inv: f64, func: CEcoreTaskCb, data: *c_void);
    fn ecore_event_handler_add(htype: c_int, func: CEcoreEventHandlerCb, 
                               data: *c_void) -> *EcoreEventHandler;
}

#[link(name = "ecore_evas")]
extern "C" {
    fn ecore_evas_init() -> c_int;
    fn ecore_evas_shutdown() -> c_int;
    fn ecore_evas_new(engine_name: *c_char, 
                      x: c_int, y: c_int, 
                      w: c_int, h: c_int,
                      extra_options: *c_char) -> *EcoreEvas;
    fn ecore_evas_show(ee: *EcoreEvas);
    fn ecore_evas_get(ee: *EcoreEvas) -> *evas::Evas;
    fn ecore_evas_data_set(ee: *EcoreEvas, key: *c_char, data: *c_void);
    fn ecore_evas_data_get(ee: *EcoreEvas, key: *c_char) -> *c_void;
    fn ecore_evas_free(ee: *EcoreEvas);
    fn ecore_evas_callback_resize_set(ee: *EcoreEvas, func: _CEcoreEvasEventCb);
    fn ecore_evas_geometry_get(ee: *EcoreEvas,
                               x: *c_int, y: *c_int,
                               w: *c_int, h: *c_int);
}

pub fn event_handler_add<T>(htype: EcoreEvent, 
                            func: EcoreEventHandlerCb<T>, 
                            data: &Option<T>) -> Box<EcoreEventHandler> {
    unsafe { 
        transmute(ecore_event_handler_add(htype as c_int, transmute(func), transmute(data)))
    }
}

pub fn init() -> i32 {
    unsafe { ecore_init() as i32 }
}

pub fn app_args_set(argc: uint, argv: Vec<String>) {
    let vchars_ptr: **c_char = to_c_args(argv);
    unsafe { ecore_app_args_set(argc as c_int, vchars_ptr) }
}

pub fn main_loop_begin() {
    unsafe { ecore_main_loop_begin() }
}

pub fn main_loop_quit() {
    unsafe { ecore_main_loop_quit() }
}

pub fn shutdown() -> int {
    unsafe { ecore_shutdown() as int }
}

pub fn time_get() -> f64 {
    unsafe { ecore_time_get() }
}

pub fn time_unix_get() -> f64 {
    unsafe { ecore_time_unix_get() }
}

pub fn timer_add<T>(inv: f64, func: EcoreTaskCb<T>, data: &Option<T>) {
    let c_data: *c_void = unsafe { transmute(data) };
    let c_func: CEcoreTaskCb = unsafe { transmute(func) };
    unsafe { ecore_timer_add(inv, c_func, c_data) }
}

pub fn evas_init() -> int {
    unsafe { ecore_evas_init() as int }
}

pub fn evas_shutdown() -> int {
    unsafe { ecore_evas_shutdown() as int }
}

// Creates a new Ecore_Evas based on engine name and common parameters.
// If 'engine_name' is None, it  use environment variable ECORE_EVAS_ENGINE,
// that can be undefined and in this case this call will try to find the
// first working engine.
pub fn evas_new(engine_name: Option<&str>,
                x: int, y: int,
                w: int, h: int,
                extra_options: &str) -> Box<EcoreEvas> {
    unsafe {
        transmute(match engine_name {
            /* Null pointer */
            None =>
                extra_options.with_c_str(|c_extra_options| {
                    ecore_evas_new(ptr::null(), x as c_int, y as c_int, 
                                   w as c_int, h as c_int, c_extra_options)
                }),
            Some(ename) =>
                ename.with_c_str(|c_engine_name| {
                    extra_options.with_c_str(|c_extra_options| {
                        ecore_evas_new(c_engine_name, x as c_int, y as c_int, 
                                       w as c_int, h as c_int, c_extra_options)
                    })
                })
        })
    }
}

/// Show an Ecore_Evas' window.
pub fn evas_show(ee: &EcoreEvas) {
    unsafe { ecore_evas_show(ee) }
}

/// Get an Ecore_Evas's Evas.
pub fn evas_get(ee: &EcoreEvas) -> Box<evas::Evas> {
    unsafe { transmute(ecore_evas_get(ee)) }
}

/// Free an Ecore_Evas.
pub fn evas_free(ee: &EcoreEvas) {
    unsafe { ecore_evas_free(ee) }
}

/// Get the geometry of an Ecore_Evas.
pub fn evas_geometry_get(ee: &EcoreEvas, x: &int, y: &int, w: &int, h: &int) {
    unsafe {
        ecore_evas_geometry_get(ee, transmute(x), transmute(y),
                                transmute(w), transmute(h))
    }
}

/// Set a callback for Ecore_Evas resize events.
pub fn evas_callback_resize_set(ee: &EcoreEvas, func: EcoreEvasEventCb) {
    unsafe {
        ecore_evas_callback_resize_set(ee, transmute(func))
    }
}

/// Store user data in an Ecore_Evas structure.
pub fn evas_data_set<T>(ee: &EcoreEvas, key: &str, data: &T) {
    key.with_c_str(|c_key| unsafe {
        ecore_evas_data_set(ee, c_key, transmute(data))
    })
}

/// Retrieve user data associated with an Ecore_Evas.
pub fn evas_data_get<T>(ee: &EcoreEvas, key: &str) -> Box<T> {
    key.with_c_str(|c_key| unsafe {
        transmute(ecore_evas_data_get(ee, c_key))
    })
}

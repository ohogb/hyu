macro_rules! implement {
    ($($children:ident),*$(,)?) => {
        pub enum Resource {
            $(
                $children(crate::wl::$children),
            )*
        }

        impl crate::wl::Object for Resource {
            fn handle(&mut self, client: &mut crate::wl::Client, op: u16, params: &[u8]) -> crate::Result<()> {
                match self {
                    $(
                        Self::$children(x) => x.handle(client, op, params),
                    )*
                }
            }
        }

        $(
            impl From<crate::wl::$children> for Resource {
                fn from(x: crate::wl::$children) -> Self {
                    Self::$children(x)
                }
            }

            impl From<Resource> for crate::Result<crate::wl::$children> {
                fn from(x: Resource) -> Self {
                    match x {
                        Resource::$children(x) => Self::Ok(x),
                        _ => Err(concat!("resource is not of type '", stringify!($children), "'"))?,
                    }
                }
            }

            impl<'a> From<&'a Resource> for crate::Result<&'a crate::wl::$children> {
                fn from(x: &'a Resource) -> Self {
                    match x {
                        Resource::$children(x) => Self::Ok(x),
                        _ => Err(concat!("resource is not of type '", stringify!($children), "'"))?,
                    }
                }
            }

            impl<'a> From<&'a mut Resource> for crate::Result<&'a mut crate::wl::$children> {
                fn from(x: &'a mut Resource) -> Self {
                    match x {
                        Resource::$children(x) => Self::Ok(x),
                        _ => Err(concat!("resource is not of type '", stringify!($children), "'"))?,
                    }
                }
            }
        )*
    }
}

implement![
	Buffer,
	Callback,
	Compositor,
	DataDevice,
	DataDeviceManager,
	Display,
	Keyboard,
	Output,
	Pointer,
	Region,
	Registry,
	Seat,
	Shm,
	ShmPool,
	SubCompositor,
	SubSurface,
	Surface,
	XdgSurface,
	XdgToplevel,
	XdgWmBase,
	ZwpLinuxDmabufV1,
];

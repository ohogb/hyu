macro_rules! implement {
    ($($children:ident),*$(,)?) => {
        pub enum Resource {
            $(
                $children(crate::wl::$children),
            )*
        }

        impl crate::wl::Object for Resource {
            fn handle(&mut self, client: &mut crate::wl::Client, op: u16, params: Vec<u8>) -> crate::Result<()> {
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
        )*
    }
}

implement![
	Buffer,
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
];

#![macro_use]

#[macro_export]
macro_rules! bit {
    ($val:expr) => {
        (1 << $val)
    };
}

#[macro_export]
macro_rules! bit_enum {
    ($name:ident ($base:ty) { $( $entry_name:ident = $entry_value:expr ),* }) => {
        #[derive(Copy, Clone, PartialEq, Eq, Debug)]
        #[repr(C)]
        pub struct $name($base);
        
        impl $name {
            pub const fn from(val: $base) -> Self {
                Self(val)
            }
            
            pub const fn contains(self, other: Self) -> bool {
                (self.0 & other.0) != 0
            }

            pub const fn get(self) -> $base {
                self.0
            }
        
            $(
                pub const fn $entry_name() -> Self {
                    Self($entry_value)
                }
            )*
        }
        
        impl const core::ops::BitOr for $name {
            type Output = Self;
        
            #[inline]
            fn bitor(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }

        impl const core::ops::BitAnd for $name {
            type Output = Self;
        
            #[inline]
            fn bitand(self, other: Self) -> Self {
                Self(self.0 & other.0)
            }
        }

        impl core::ops::BitOrAssign for $name {
            #[inline]
            fn bitor_assign(&mut self, other: Self) {
                self.0 |= other.0
            }
        }
        
        impl core::ops::BitAndAssign for $name {
            #[inline]
            fn bitand_assign(&mut self, other: Self) {
                self.0 &= other.0
            }
        }
    };
}

#[macro_export]
macro_rules! bit_group {
    ($base:ty [ $( $val:ident ),* ]) => {
        <$base>::from( $( <$base>::$val().get() )|* )
    };
}

#[macro_export]
macro_rules! util_return_if {
    ($cond:expr, $ret:expr) => {
        if $cond {
            return $ret;
        }
    }
}

#[macro_export]
macro_rules! util_return_unless {
    ($cond:expr, $ret:expr) => {
        if !$cond {
            return $ret;
        }
    }
}

#[macro_export]
macro_rules! write_bits {
    ($start:expr, $end:expr, $value:expr, $data:expr) => {
        $value = ($value & (!( ((1 << ($end - $start + 1)) - 1) << $start ))) | ($data << $start);
    };
}

#[macro_export]
macro_rules! read_bits {
    ($start:expr, $end:expr, $value:expr) => {
        ($value & (((1 << ($end - $start + 1)) - 1) << $start)) >> $start
    };
}

#[macro_export]
macro_rules! nul {
    ($lit:expr) => {
        concat!($lit, "\0\0\0\0\0\0\0\0")
    };
}
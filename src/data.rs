/*
 * Copyright 2017 icasdri
 *
 * This file is part of stabping. The original source code for stabping can be
 * found at <https://github.com/icasdri/stabping>. See COPYING for licensing
 * details.
 */

use std::mem;
use std::slice;
use std::collections::BTreeSet;
use std::cmp::Ordering;

/**
 * Trait for extracting the bytes (as a u8 slice) out of any Sized value.
 */
pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

impl<T> AsBytes for T where T: Sized {
    fn as_bytes(&self) -> &[u8] {
        let orig_ptr: *const T = self;
        let raw = orig_ptr as *const u8;
        let len = mem::size_of::<T>();
        unsafe {
            slice::from_raw_parts(raw, len)
        }
    }
}

/**
 * Trait for generalizing over different representations of data elements
 * on-disk.
 */
pub trait DataElement {
    fn get_time(&self) -> u32;
    fn get_index(&self) -> u32;
    fn size_of_data_vals(&self) -> usize;
    fn data_vals_as_bytes(&self) -> &[u8];
}

/**
 * Representation of data elements on-disk for raw (non-averaged) collections.
 */
#[repr(C, packed)]
pub struct DiscreteDataOnDisk {
    time: u32,
    index: u32,
    val: f32,  // the raw value
}

/**
 * Representation of data elements on-disk for averaged collections.
 */
#[repr(C, packed)]
pub struct AveragedDataOnDisk {
    time: u32,
    index: u32,
    val_sd: [f32; 2],  // [averaged value, standard deviation]
}

impl DataElement for DiscreteDataOnDisk {
    fn get_time(&self) -> u32 { self.time }
    fn get_index(&self) -> u32 { self.index }
    fn size_of_data_vals(&self) -> usize { mem::size_of_val(&self.val) }
    fn data_vals_as_bytes(&self) -> &[u8] { self.val.as_bytes() }
}

impl DataElement for AveragedDataOnDisk {
    fn get_time(&self) -> u32 { self.time }
    fn get_index(&self) -> u32 { self.index }
    fn size_of_data_vals(&self) -> usize { mem::size_of_val(&self.val_sd) }
    fn data_vals_as_bytes(&self) -> &[u8] { self.val_sd.as_bytes() }
}

/*
 * Ord (and thus Eq, PartialEq, and PartialOrd) implementation for DataElement
 * over their indices (via get_index()) so that they can be put in BTreeSets.
 */
impl Ord for DiscreteDataOnDisk {
    fn cmp(&self, other: &DiscreteDataOnDisk) -> Ordering {
        self.get_index().cmp(&other.get_index())
    }
}

impl PartialOrd for DiscreteDataOnDisk {
    fn partial_cmp(&self, other: &DiscreteDataOnDisk) -> Option<Ordering> {
        self.get_index().partial_cmp(&other.get_index())
    }
}

impl PartialEq for DiscreteDataOnDisk {
    fn eq(&self, other: &DiscreteDataOnDisk) -> bool {
        self.get_index() == other.get_index()
    }
}

impl Eq for DiscreteDataOnDisk {}

impl Ord for AveragedDataOnDisk {
    fn cmp(&self, other: &AveragedDataOnDisk) -> Ordering {
        self.get_index().cmp(&other.get_index())
    }
}

impl PartialOrd for AveragedDataOnDisk {
    fn partial_cmp(&self, other: &AveragedDataOnDisk) -> Option<Ordering> {
        self.get_index().partial_cmp(&other.get_index())
    }
}

impl PartialEq for AveragedDataOnDisk {
    fn eq(&self, other: &AveragedDataOnDisk) -> bool {
        self.get_index() == other.get_index()
    }
}

impl Eq for AveragedDataOnDisk {}


pub enum ToWireError {
    IncompatibleTimes
}

pub trait ToWire {
    fn to_wire(&self, wire: &mut Vec<u8>) -> Result<(), ToWireError>;
    fn space_necessary(&self) -> usize;
}

impl<T> ToWire for BTreeSet<T> where T: DataElement + Ord {
    fn to_wire(&self, wire: &mut Vec<u8>) -> Result<(), ToWireError> {
        let mut time = None;

        for d in self.iter() {
            match (time, d.get_time()) {
                (None, t) => {
                    time = Some(t)
                }
                (Some(e), t) if e == t => {},
                _ => {
                    return Err(ToWireError::IncompatibleTimes);
                }
            }
            wire.extend_from_slice(d.data_vals_as_bytes());
        }

        Ok(())
    }

    fn space_necessary(&self) -> usize {
        self.len() * mem::size_of::<T>()
    }
}
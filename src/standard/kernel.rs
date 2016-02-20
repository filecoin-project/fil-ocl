//! An OpenCL kernel.

use std::convert::Into;
use std::collections::HashMap;
use raw::{self, OclNum, Kernel as KernelRaw, CommandQueue as CommandQueueRaw, KernelArg};
use error::{Result as OclResult, Error as OclError};
use standard::{WorkDims, Buffer, EventList, Program, Queue};

/// A kernel.
///
/// # Destruction
/// Releases kernel object automatically upon drop.
///
/// # Thread Safety
///
/// Do not share the kernel object pointer `obj` between threads. 
/// Specifically, do not attempt to create or modify kernel arguments
/// from more than one thread for a kernel.
///
/// TODO: Add more details, examples, etc.
/// TODO: Add information about panics and errors.
#[derive(Debug)]
pub struct Kernel {
    obj_raw: KernelRaw,
    name: String,
    arg_index: u32,
    named_args: HashMap<&'static str, u32>,
    arg_count: u32,
    command_queue: CommandQueueRaw,
    gwo: WorkDims,
    gws: WorkDims,
    lws: WorkDims,
}

impl Kernel {
    /// Returns a new kernel.
    // TODO: Implement proper error handling (return result etc.).
    pub fn new<S: Into<String>>(name: S, program: &Program, queue: &Queue, 
                gws: WorkDims ) -> OclResult<Kernel>
    {
        let name = name.into();
        let obj_raw = try!(raw::create_kernel(program.raw_as_ref(), &name));

        Ok(Kernel {
            obj_raw: obj_raw,
            name: name,
            arg_index: 0,
            named_args: HashMap::with_capacity(5),
            arg_count: 0u32,
            command_queue: queue.raw_as_ref().clone(),
            gwo: WorkDims::Unspecified,
            gws: gws,
            lws: WorkDims::Unspecified,
        })
    }

    /// Sets the global work offset (builder-style).
    pub fn gwo(mut self, gwo: WorkDims) -> Kernel {
        if gwo.dim_count() == self.gws.dim_count() {
            self.gwo = gwo
        } else {
            panic!("ocl::Kernel::gwo(): Work size mismatch.");
        }
        self
    }

    /// Sets the local work size (builder-style).
    pub fn lws(mut self, lws: WorkDims) -> Kernel {
        if lws.dim_count() == self.gws.dim_count() {
            self.lws = lws;
        } else {
            panic!("ocl::Kernel::lws(): Work size mismatch.");
        }
        self
    }

    /// Adds a new argument to the kernel specifying the buffer object represented
    /// by 'buffer' (builder-style). Argument is added to the bottom of the argument 
    /// order.
    pub fn arg_buf<T: OclNum>(mut self, buffer: &Buffer<T>) -> Kernel {
        self.new_arg_buf(Some(buffer));
        self
    }

    /// Adds a new argument specifying the value: `scalar` (builder-style). Argument 
    /// is added to the bottom of the argument order.
    pub fn arg_scl<T: OclNum>(mut self, scalar: T) -> Kernel {
        self.new_arg_scl(Some(scalar));
        self
    }

    /// Adds a new argument specifying the allocation of a local variable of size
    /// `length * sizeof(T)` bytes (builder_style).
    ///
    /// Local variables are used to share data between work items in the same 
    /// workgroup.
    pub fn arg_loc<T: OclNum>(mut self, length: usize) -> Kernel {
        self.new_arg_loc::<T>(length);
        self
    }

    /// Adds a new named argument (in order) specifying the value: `scalar` 
    /// (builder-style).
    ///
    /// Named arguments can be easily modified later using `::set_arg_scl_named()`.
    pub fn arg_scl_named<T: OclNum>(mut self, name: &'static str, scalar_opt: Option<T>) -> Kernel {
        let arg_idx = self.new_arg_scl(scalar_opt);
        self.named_args.insert(name, arg_idx);
        self
    }

    /// Adds a new named buffer argument specifying the buffer object represented by 
    /// 'buffer' (builder-style). Argument is added to the bottom of the argument order.
    ///
    /// Named arguments can be easily modified later using `::set_arg_scl_named()`.
    pub fn arg_buf_named<T: OclNum>(mut self, name: &'static str,  buffer_opt: Option<&Buffer<T>>) -> Kernel {
        let arg_idx = self.new_arg_buf(buffer_opt);
        self.named_args.insert(name, arg_idx);

        self
    }     

    /// Modifies the kernel argument named: `name`.
    // [FIXME]: CHECK THAT NAME EXISTS AND GIVE A BETTER ERROR MESSAGE
    pub fn set_arg_scl_named<T: OclNum>(&mut self, name: &'static str, scalar: T) 
            -> OclResult<()> 
    {
        let arg_idx = try!(self.resolve_named_arg_idx(name));
        self.set_arg::<T>(arg_idx, KernelArg::Scalar(&scalar))
    }

    /// Modifies the kernel argument named: `name`.
    // [FIXME] TODO: CHECK THAT NAME EXISTS AND GIVE A BETTER ERROR MESSAGE
    pub fn set_arg_buf_named<T: OclNum>(&mut self, name: &'static str, 
                buffer_opt: Option<&Buffer<T>>)  -> OclResult<()>   
    {
        //  TODO: ADD A CHECK FOR A VALID NAME (KEY)
        // let arg_idx = self.named_args[name];
        let arg_idx = try!(self.resolve_named_arg_idx(name));

        match buffer_opt {
            Some(buffer) => {
                self.set_arg::<T>(arg_idx, KernelArg::Mem(buffer.raw_as_ref()))
            },
            None => {
                // let mem_raw_null = unsafe { MemRaw::null() };
                self.set_arg::<T>(arg_idx, KernelArg::MemNull)
            },
        }
    }

    fn resolve_named_arg_idx(&self, name: &'static str) -> OclResult<u32> {
        match self.named_args.get(name) {
            Some(&ai) => Ok(ai),
            None => {
                OclError::err(format!("Kernel::set_arg_scl_named(): Invalid argument \
                    name: '{}'.", name))
            },
        }
    }

    /// Enqueues kernel on the default command queue.
    ///
    /// TODO: Implement 'alternative queue' version of this function.
    #[inline]
    pub fn enqueue(&self, wait_list: Option<&EventList>, dest_list: Option<&mut EventList>) {
        raw::enqueue_kernel(&self.command_queue, &self.obj_raw, self.gws.dim_count(), 
            self.gwo.as_raw(), self.gws.as_raw().unwrap(), self.lws.as_raw(), 
            wait_list.map(|el| el.raw_as_ref()), dest_list.map(|el| el.raw_as_mut()), Some(&self.name))
            .unwrap();
    }

    /// Returns the number of arguments specified for this kernel.
    #[inline]
    pub fn arg_count(&self) -> u32 {
        self.arg_count
    }    

    // Non-builder-style version of `::arg_buf()`.
    fn new_arg_buf<T: OclNum>(&mut self, buffer_opt: Option<&Buffer<T>>) -> u32 {        
        // This value lives long enough to be copied by `clSetKernelArg`.
        // let buf_obj = match buffer_opt {
        //     Some(buffer) => buffer.raw_as_ref(),
        //     None => unsafe { MemRaw::null() },
        // };

        // self.new_arg::<T>(KernelArg::Mem(&buf_obj))

        match buffer_opt {
            Some(buffer) => {
                self.new_arg::<T>(KernelArg::Mem(buffer.raw_as_ref()))
            },
            None => {
                // let mem_raw_null = unsafe { MemRaw::null() };
                self.new_arg::<T>(KernelArg::MemNull)
            },
        }
    }

    // Non-builder-style version of `::arg_scl()`.
    fn new_arg_scl<T: OclNum>(&mut self, scalar_opt: Option<T>) -> u32 {
        let scalar = match scalar_opt {
            Some(scl) => scl,
            None => Default::default(),
        };

        self.new_arg::<T>(KernelArg::Scalar(&scalar))
    }

    // Non-builder-style version of `::arg_loc()`.
    //
    // `length` lives long enough to be copied by `clSetKernelArg`.
    fn new_arg_loc<T: OclNum>(&mut self, length: usize) -> u32 {
        self.new_arg::<T>(KernelArg::Local(&length))
    } 

    // Adds a new argument to the kernel and returns the index.
    fn new_arg<T: OclNum>(&mut self, arg: KernelArg<T>) -> u32 {
        let arg_idx = self.arg_index;

        raw::set_kernel_arg::<T>(&self.obj_raw, arg_idx, 
            arg,
            Some(&self.name)
        ).unwrap();

        self.arg_index += 1;
        arg_idx
    } 

    fn set_arg<T: OclNum>(&self, arg_idx: u32, arg: KernelArg<T>) -> OclResult<()> {
        raw::set_kernel_arg::<T>(&self.obj_raw, arg_idx, arg, Some(&self.name))
    }

    pub fn raw_as_ref(&self) -> &KernelRaw {
        &self.obj_raw
    }
}

// impl Drop for Kernel {
//     fn drop(&mut self) {
//         // println!("DROPPING KERNEL");
//         raw::release_kernel(self.obj_raw).unwrap();
//     }
// }

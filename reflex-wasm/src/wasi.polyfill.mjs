// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
export default class WASI {
  static ERRNO_ESUCCESS = 0;
  static ERRNO_EBADF = 8;
  static ERRNO_EINVAL = 28;
  static ERRNO_ENOSYS = 52;

  static FD_STDIN = 0;
  static FD_STDOUT = 1;
  static FD_STDERR = 1;

  constructor() {
    this.instance = null;
    this.wasiImport = {
      args_get: args_get.bind(this),
      args_sizes_get: args_sizes_get.bind(this),
      environ_get: environ_get.bind(this),
      environ_sizes_get: environ_sizes_get.bind(this),
      fd_close: fd_close.bind(this),
      fd_fdstat_get: fd_fdstat_get.bind(this),
      fd_prestat_dir_name: fd_prestat_dir_name.bind(this),
      fd_prestat_get: fd_prestat_get.bind(this),
      fd_seek: fd_seek.bind(this),
      fd_write: fd_write.bind(this),
      poll_oneoff: poll_oneoff.bind(this),
      proc_exit: proc_exit.bind(this),
    };

    function fd_prestat_get(fd, bufPtr) {
      return WASI.ERRNO_EBADF;
    }

    function fd_prestat_dir_name(fd, pathPtr, pathLen) {
      return WASI.ERRNO_EINVAL;
    }

    function environ_sizes_get(environCount, environBufSize) {
      const view = new DataView(this.getModuleMemory());
      view.setUint32(environCount, 0, true);
      view.setUint32(environBufSize, 0, true);
      return WASI.ERRNO_ESUCCESS;
    }

    function environ_get(environ, environBuf) {
      return WASI.ERRNO_ESUCCESS;
    }

    function args_sizes_get(argc, argvBufSize) {
      const view = new DataView(this.getModuleMemory());
      view.setUint32(argc, 0, true);
      view.setUint32(argvBufSize, 0, true);
      return WASI.ERRNO_ESUCCESS;
    }

    function args_get(argv, argvBuf) {
      return WASI.ERRNO_ESUCCESS;
    }

    function fd_fdstat_get(fd, bufPtr) {
      const view = new DataView(this.getModuleMemory());
      view.setUint8(bufPtr, fd);
      view.setUint16(bufPtr + 2, 0, true);
      view.setUint16(bufPtr + 4, 0, true);
      setBigUint64(view, bufPtr + 8, 0, true);
      setBigUint64(view, bufPtr + 8 + 8, 0, true);
      return WASI.ERRNO_ESUCCESS;

      function setBigUint64(view, byteOffset, value, littleEndian) {
        const lowWord = value;
        const highWord = 0;
        view.setUint32(byteOffset + littleEndian ? 0 : 4, lowWord, littleEndian);
        view.setUint32(byteOffset + littleEndian ? 4 : 0, highWord, littleEndian);
      }
    }

    function fd_write(fd, iovs, iovsLen, nwritten) {
      const memory = this.getModuleMemory();
      const view = new DataView(memory);
      function getiovs(iovs, iovsLen) {
        const buffers = Array.from({ length: iovsLen }, (_, i) => {
          const ptr = iovs + i * 8;
          const buf = view.getUint32(ptr, true);
          const bufLen = view.getUint32(ptr + 4, true);
          return new Uint8Array(memory, buf, bufLen);
        });
        return buffers;
      }
      const buffers = getiovs(iovs, iovsLen);
      const bytes = buffers.reduce((bytes, iov) => {
        bytes.push(...iov);
        return bytes;
      }, []);
      if (fd === WASI.FD_STDOUT) console.log(String.fromCharCode.apply(null, bytes));
      view.setUint32(nwritten, bytes.length, true);
      return WASI.ERRNO_ESUCCESS;
    }

    function poll_oneoff(sin, sout, nsubscriptions, nevents) {
      return WASI.ERRNO_ENOSYS;
    }

    function proc_exit(rval) {
      return WASI.ERRNO_ENOSYS;
    }

    function fd_close(fd) {
      return WASI.ERRNO_ENOSYS;
    }

    function fd_seek(fd, offset, whence, newOffsetPtr) {}

    function fd_close(fd) {
      return WASI.ERRNO_ENOSYS;
    }
  }
  initialize(instance) {
    this.init(instance);
    const start = instance.exports._start;
    const initialize = instance.exports._initialize;
    if (typeof start === 'function' && typeof initialize === 'function') {
      throw new Error('WASI modules cannot export both _start() and _initialize() functions');
    } else if (typeof initialize === 'function') {
      initialize();
    } else {
      throw new Error('Missing WASI module "_initialize" function export');
    }
  }
  start(instance) {
    this.init(instance);
    const start = instance.exports._start;
    const initialize = instance.exports._initialize;
    if (typeof start === 'function' && typeof initialize === 'function') {
      throw new Error('WASI modules cannot export both _start() and _initialize() functions');
    } else if (typeof start === 'function') {
      start();
    } else {
      throw new Error('Missing WASI module "_start" function export');
    }
  }
  init(instance) {
    if (!(instance.exports.memory instanceof WebAssembly.Memory)) {
      throw new Error('WASI modules must export a memory named "memory"');
    }
    if (!(instance.exports.__indirect_function_table instanceof WebAssembly.Table)) {
      throw new Error('WASI modules must export a table named "__indirect_function_table"');
    }
    this.instance = instance;
  }
  getModuleMemory() {
    return this.instance.exports.memory.buffer;
  }
}

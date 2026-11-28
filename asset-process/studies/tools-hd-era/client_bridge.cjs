var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __esm = (fn, res, err) => function __init() {
  if (err) throw err[0];
  try {
    return fn && (res = (0, fn[__getOwnPropNames(fn)[0]])(fn = 0)), res;
  } catch (e) {
    throw err = [e], e;
  }
};
var __commonJS = (cb, mod) => function __require() {
  try {
    return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
  } catch (e) {
    throw mod = 0, e;
  }
};
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// {TSPS_CHECKOUT}/client/common/utils/FloatUtil.ts
var FloatUtil;
var init_FloatUtil = __esm({
  "{TSPS_CHECKOUT}/client/common/utils/FloatUtil.ts"() {
    "use strict";
    FloatUtil = class _FloatUtil {
      static {
        this.MAX_VALUE = 34028234663852886e22;
      }
      static {
        this.float = new Float32Array(1);
      }
      static {
        this.integer = new Int32Array(_FloatUtil.float.buffer);
      }
      static floatBitsToInt(n) {
        _FloatUtil.float[0] = n;
        return _FloatUtil.integer[0];
      }
      static intBitsToFloat(n) {
        _FloatUtil.integer[0] = n;
        return _FloatUtil.float[0];
      }
      static packFloat11(v) {
        return 1024 - Math.round(v / (1 / 64));
      }
      static unpackFloat11(v) {
        return 16 - v / 64;
      }
      // 0-1, 1/63 decimal precision
      static packFloat6(v) {
        return Math.round(v / (1 / 63));
      }
      static unpackFloat6(v) {
        return v / 63;
      }
    };
  }
});

// {TSPS_CHECKOUT}/client/rs/io/ByteBuffer.ts
var ByteBuffer;
var init_ByteBuffer = __esm({
  "{TSPS_CHECKOUT}/client/rs/io/ByteBuffer.ts"() {
    "use strict";
    init_FloatUtil();
    ByteBuffer = class {
      constructor(dataOrSize) {
        this.offset = 0;
        if (dataOrSize instanceof Int8Array) {
          this._data = dataOrSize;
        } else if (dataOrSize instanceof Uint8Array) {
          this._data = new Int8Array(
            dataOrSize.buffer,
            dataOrSize.byteOffset,
            dataOrSize.byteLength
          );
        } else if (dataOrSize instanceof ArrayBuffer) {
          this._data = new Int8Array(dataOrSize);
        } else {
          this._data = new Int8Array(dataOrSize);
        }
      }
      readByte() {
        if (this.offset > this._data.length - 1) {
          throw new Error("Buffer overflow");
        }
        return this._data[this.offset++];
      }
      readUnsignedByte() {
        return this.readByte() & 255;
      }
      readShort() {
        return (this.readUnsignedByte() << 8 | this.readUnsignedByte()) << 16 >> 16;
      }
      readUnsignedShort() {
        return this.readShort() & 65535;
      }
      // cg2
      readSignedShort() {
        const v = this.readUnsignedShort();
        if (v > 32767) {
          return v - 65536;
        }
        return v;
      }
      readMedium() {
        return this.readUnsignedByte() << 16 | this.readUnsignedByte() << 8 | this.readUnsignedByte();
      }
      readUnsignedMedium() {
        return this.readMedium() & 16777215;
      }
      /** A signed 24-bit value (g3s): 0xFFFFFF reads as -1. */
      readSignedMedium() {
        return this.readMedium() << 8 >> 8;
      }
      readInt() {
        return this.readUnsignedByte() << 24 | this.readUnsignedByte() << 16 | this.readUnsignedByte() << 8 | this.readUnsignedByte();
      }
      readFloat() {
        return FloatUtil.intBitsToFloat(this.readInt());
      }
      readBigSmart() {
        if (this.getByte(this.offset) < 0) {
          return this.readInt() & 2147483647;
        } else {
          const v = this.readUnsignedShort();
          if (v === 32767) {
            return -1;
          }
          return v;
        }
      }
      readUnsignedSmart() {
        if (this.getUnsignedByte(this.offset) < 128) {
          return this.readUnsignedByte();
        } else {
          return this.readUnsignedShort() - 32768;
        }
      }
      readUnsignedSmartMin1() {
        if (this.getUnsignedByte(this.offset) < 128) {
          return this.readUnsignedByte() - 1;
        } else {
          return this.readUnsignedShort() - 32769;
        }
      }
      readSmart2() {
        if (this.getByte(this.offset) >= 0) {
          return this.readUnsignedByte() - 64;
        } else {
          return this.readUnsignedShort() - 49152;
        }
      }
      readSmart3() {
        let i = 0;
        let i_33_ = this.readUnsignedSmart();
        while (i_33_ === 32767) {
          i_33_ = this.readUnsignedSmart();
          i += 32767;
        }
        i += i_33_;
        return i;
      }
      readUnsignedShortSmart() {
        const peek = this.getUnsignedByte(this.offset);
        if (peek < 128) {
          return this.readUnsignedByte();
        }
        return this.readUnsignedShort() - 32768;
      }
      readUnsignedShortSmartMinusOne() {
        const peek = this.getUnsignedByte(this.offset);
        if (peek < 128) {
          return this.readUnsignedByte() - 1;
        }
        return this.readUnsignedShort() - 32769;
      }
      readVarInt() {
        let value = 0;
        while (true) {
          const b = this.readUnsignedByte();
          if (b & 128) {
            value = (value | b & 127) << 7;
          } else {
            return value | b;
          }
        }
      }
      readVarInt2() {
        let value = 0;
        let shift = 0;
        while (true) {
          const b = this.readUnsignedByte();
          value |= (b & 127) << shift;
          if (b <= 127) {
            break;
          }
          shift += 7;
        }
        return value;
      }
      readString(endValue = 0) {
        let str = "";
        while (this.getByte(this.offset) !== endValue) {
          str += String.fromCharCode(this.readUnsignedByte());
        }
        this.readByte();
        return str;
      }
      peek(offsetDelta = 0) {
        return this.getUnsignedByte(this.offset + offsetDelta);
      }
      readNullString() {
        if (this.getByte(this.offset) === 0) {
          this.offset++;
          return void 0;
        } else {
          return this.readString();
        }
      }
      readVerString() {
        if (this.readByte() !== 0) {
          return void 0;
        }
        return this.readString();
      }
      getByte(offset) {
        return this._data[offset];
      }
      getUnsignedByte(offset) {
        return this.getByte(offset) & 255;
      }
      getShort(offset) {
        return this.getUnsignedByte(offset) << 8 | this.getUnsignedByte(offset + 1);
      }
      getUnsignedShort(offset) {
        return this.getShort(offset) & 65535;
      }
      getInt(offset) {
        return this.getUnsignedByte(offset) << 24 | this.getUnsignedByte(offset + 1) << 16 | this.getUnsignedByte(offset + 2) << 8 | this.getUnsignedByte(offset + 3);
      }
      readBytes(amount) {
        const bytes = this._data.subarray(this.offset, this.offset + amount);
        this.offset += amount;
        return bytes;
      }
      readUnsignedBytes(amount) {
        const bytes = new Uint8Array(this._data.buffer, this._data.byteOffset, this._data.byteLength).subarray(this.offset, this.offset + amount);
        this.offset += amount;
        return bytes;
      }
      writeBytes(bytes) {
        this._data.set(bytes, this.offset);
        this.offset += bytes.length;
      }
      writeInt(v) {
        this._data[this.offset++] = v >> 24;
        this._data[this.offset++] = v >> 16;
        this._data[this.offset++] = v >> 8;
        this._data[this.offset++] = v;
      }
      setInt(offset, v) {
        this._data[offset++] = v >> 24;
        this._data[offset++] = v >> 16;
        this._data[offset++] = v >> 8;
        this._data[offset++] = v;
      }
      get length() {
        return this._data.length;
      }
      get remaining() {
        return this.length - this.offset;
      }
      get data() {
        return this._data;
      }
    };
  }
});

// {TSPS_CHECKOUT}/client/node_modules/denque/index.js
var require_denque = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/denque/index.js"(exports2, module2) {
    "use strict";
    function Denque3(array, options) {
      var options = options || {};
      this._capacity = options.capacity;
      this._head = 0;
      this._tail = 0;
      if (Array.isArray(array)) {
        this._fromArray(array);
      } else {
        this._capacityMask = 3;
        this._list = new Array(4);
      }
    }
    Denque3.prototype.peekAt = function peekAt(index) {
      var i = index;
      if (i !== (i | 0)) {
        return void 0;
      }
      var len = this.size();
      if (i >= len || i < -len) return void 0;
      if (i < 0) i += len;
      i = this._head + i & this._capacityMask;
      return this._list[i];
    };
    Denque3.prototype.get = function get(i) {
      return this.peekAt(i);
    };
    Denque3.prototype.peek = function peek() {
      if (this._head === this._tail) return void 0;
      return this._list[this._head];
    };
    Denque3.prototype.peekFront = function peekFront() {
      return this.peek();
    };
    Denque3.prototype.peekBack = function peekBack() {
      return this.peekAt(-1);
    };
    Object.defineProperty(Denque3.prototype, "length", {
      get: function length() {
        return this.size();
      }
    });
    Denque3.prototype.size = function size() {
      if (this._head === this._tail) return 0;
      if (this._head < this._tail) return this._tail - this._head;
      else return this._capacityMask + 1 - (this._head - this._tail);
    };
    Denque3.prototype.unshift = function unshift(item) {
      if (arguments.length === 0) return this.size();
      var len = this._list.length;
      this._head = this._head - 1 + len & this._capacityMask;
      this._list[this._head] = item;
      if (this._tail === this._head) this._growArray();
      if (this._capacity && this.size() > this._capacity) this.pop();
      if (this._head < this._tail) return this._tail - this._head;
      else return this._capacityMask + 1 - (this._head - this._tail);
    };
    Denque3.prototype.shift = function shift() {
      var head = this._head;
      if (head === this._tail) return void 0;
      var item = this._list[head];
      this._list[head] = void 0;
      this._head = head + 1 & this._capacityMask;
      if (head < 2 && this._tail > 1e4 && this._tail <= this._list.length >>> 2) this._shrinkArray();
      return item;
    };
    Denque3.prototype.push = function push(item) {
      if (arguments.length === 0) return this.size();
      var tail = this._tail;
      this._list[tail] = item;
      this._tail = tail + 1 & this._capacityMask;
      if (this._tail === this._head) {
        this._growArray();
      }
      if (this._capacity && this.size() > this._capacity) {
        this.shift();
      }
      if (this._head < this._tail) return this._tail - this._head;
      else return this._capacityMask + 1 - (this._head - this._tail);
    };
    Denque3.prototype.pop = function pop() {
      var tail = this._tail;
      if (tail === this._head) return void 0;
      var len = this._list.length;
      this._tail = tail - 1 + len & this._capacityMask;
      var item = this._list[this._tail];
      this._list[this._tail] = void 0;
      if (this._head < 2 && tail > 1e4 && tail <= len >>> 2) this._shrinkArray();
      return item;
    };
    Denque3.prototype.removeOne = function removeOne(index) {
      var i = index;
      if (i !== (i | 0)) {
        return void 0;
      }
      if (this._head === this._tail) return void 0;
      var size = this.size();
      var len = this._list.length;
      if (i >= size || i < -size) return void 0;
      if (i < 0) i += size;
      i = this._head + i & this._capacityMask;
      var item = this._list[i];
      var k;
      if (index < size / 2) {
        for (k = index; k > 0; k--) {
          this._list[i] = this._list[i = i - 1 + len & this._capacityMask];
        }
        this._list[i] = void 0;
        this._head = this._head + 1 + len & this._capacityMask;
      } else {
        for (k = size - 1 - index; k > 0; k--) {
          this._list[i] = this._list[i = i + 1 + len & this._capacityMask];
        }
        this._list[i] = void 0;
        this._tail = this._tail - 1 + len & this._capacityMask;
      }
      return item;
    };
    Denque3.prototype.remove = function remove(index, count) {
      var i = index;
      var removed;
      var del_count = count;
      if (i !== (i | 0)) {
        return void 0;
      }
      if (this._head === this._tail) return void 0;
      var size = this.size();
      var len = this._list.length;
      if (i >= size || i < -size || count < 1) return void 0;
      if (i < 0) i += size;
      if (count === 1 || !count) {
        removed = new Array(1);
        removed[0] = this.removeOne(i);
        return removed;
      }
      if (i === 0 && i + count >= size) {
        removed = this.toArray();
        this.clear();
        return removed;
      }
      if (i + count > size) count = size - i;
      var k;
      removed = new Array(count);
      for (k = 0; k < count; k++) {
        removed[k] = this._list[this._head + i + k & this._capacityMask];
      }
      i = this._head + i & this._capacityMask;
      if (index + count === size) {
        this._tail = this._tail - count + len & this._capacityMask;
        for (k = count; k > 0; k--) {
          this._list[i = i + 1 + len & this._capacityMask] = void 0;
        }
        return removed;
      }
      if (index === 0) {
        this._head = this._head + count + len & this._capacityMask;
        for (k = count - 1; k > 0; k--) {
          this._list[i = i + 1 + len & this._capacityMask] = void 0;
        }
        return removed;
      }
      if (i < size / 2) {
        this._head = this._head + index + count + len & this._capacityMask;
        for (k = index; k > 0; k--) {
          this.unshift(this._list[i = i - 1 + len & this._capacityMask]);
        }
        i = this._head - 1 + len & this._capacityMask;
        while (del_count > 0) {
          this._list[i = i - 1 + len & this._capacityMask] = void 0;
          del_count--;
        }
        if (index < 0) this._tail = i;
      } else {
        this._tail = i;
        i = i + count + len & this._capacityMask;
        for (k = size - (count + index); k > 0; k--) {
          this.push(this._list[i++]);
        }
        i = this._tail;
        while (del_count > 0) {
          this._list[i = i + 1 + len & this._capacityMask] = void 0;
          del_count--;
        }
      }
      if (this._head < 2 && this._tail > 1e4 && this._tail <= len >>> 2) this._shrinkArray();
      return removed;
    };
    Denque3.prototype.splice = function splice(index, count) {
      var i = index;
      if (i !== (i | 0)) {
        return void 0;
      }
      var size = this.size();
      if (i < 0) i += size;
      if (i > size) return void 0;
      if (arguments.length > 2) {
        var k;
        var temp;
        var removed;
        var arg_len = arguments.length;
        var len = this._list.length;
        var arguments_index = 2;
        if (!size || i < size / 2) {
          temp = new Array(i);
          for (k = 0; k < i; k++) {
            temp[k] = this._list[this._head + k & this._capacityMask];
          }
          if (count === 0) {
            removed = [];
            if (i > 0) {
              this._head = this._head + i + len & this._capacityMask;
            }
          } else {
            removed = this.remove(i, count);
            this._head = this._head + i + len & this._capacityMask;
          }
          while (arg_len > arguments_index) {
            this.unshift(arguments[--arg_len]);
          }
          for (k = i; k > 0; k--) {
            this.unshift(temp[k - 1]);
          }
        } else {
          temp = new Array(size - (i + count));
          var leng = temp.length;
          for (k = 0; k < leng; k++) {
            temp[k] = this._list[this._head + i + count + k & this._capacityMask];
          }
          if (count === 0) {
            removed = [];
            if (i != size) {
              this._tail = this._head + i + len & this._capacityMask;
            }
          } else {
            removed = this.remove(i, count);
            this._tail = this._tail - leng + len & this._capacityMask;
          }
          while (arguments_index < arg_len) {
            this.push(arguments[arguments_index++]);
          }
          for (k = 0; k < leng; k++) {
            this.push(temp[k]);
          }
        }
        return removed;
      } else {
        return this.remove(i, count);
      }
    };
    Denque3.prototype.clear = function clear() {
      this._list = new Array(this._list.length);
      this._head = 0;
      this._tail = 0;
    };
    Denque3.prototype.isEmpty = function isEmpty() {
      return this._head === this._tail;
    };
    Denque3.prototype.toArray = function toArray() {
      return this._copyArray(false);
    };
    Denque3.prototype._fromArray = function _fromArray(array) {
      var length = array.length;
      var capacity = this._nextPowerOf2(length);
      this._list = new Array(capacity);
      this._capacityMask = capacity - 1;
      this._tail = length;
      for (var i = 0; i < length; i++) this._list[i] = array[i];
    };
    Denque3.prototype._copyArray = function _copyArray(fullCopy, size) {
      var src = this._list;
      var capacity = src.length;
      var length = this.length;
      size = size | length;
      if (size == length && this._head < this._tail) {
        return this._list.slice(this._head, this._tail);
      }
      var dest = new Array(size);
      var k = 0;
      var i;
      if (fullCopy || this._head > this._tail) {
        for (i = this._head; i < capacity; i++) dest[k++] = src[i];
        for (i = 0; i < this._tail; i++) dest[k++] = src[i];
      } else {
        for (i = this._head; i < this._tail; i++) dest[k++] = src[i];
      }
      return dest;
    };
    Denque3.prototype._growArray = function _growArray() {
      if (this._head != 0) {
        var newList = this._copyArray(true, this._list.length << 1);
        this._tail = this._list.length;
        this._head = 0;
        this._list = newList;
      } else {
        this._tail = this._list.length;
        this._list.length <<= 1;
      }
      this._capacityMask = this._capacityMask << 1 | 1;
    };
    Denque3.prototype._shrinkArray = function _shrinkArray() {
      this._list.length >>>= 1;
      this._capacityMask >>>= 1;
    };
    Denque3.prototype._nextPowerOf2 = function _nextPowerOf2(num) {
      var log2 = Math.log(num) / Math.log(2);
      var nextPow2 = 1 << log2 + 1;
      return Math.max(nextPow2, 4);
    };
    module2.exports = Denque3;
  }
});

// {TSPS_CHECKOUT}/client/node_modules/java-random/lib/index.js
var require_lib = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/java-random/lib/index.js"(exports2, module2) {
    var p2_16 = 65536;
    var p2_24 = 16777216;
    var p2_27 = 134217728;
    var p2_31 = 2147483648;
    var p2_32 = 4294967296;
    var p2_48 = 281474976710656;
    var p2_53 = Math.pow(2, 53);
    var m2_16 = 65535;
    var [c2, c1, c0] = [5, 57068, 58989];
    module2.exports = class JavaRandom {
      constructor(seedval) {
        let s2, s1, s0;
        let nextNextGaussian;
        let haveNextNextGaussian = false;
        const _next = () => {
          let carry = 11;
          let r0 = s0 * c0 + carry;
          carry = r0 >>> 16;
          r0 &= m2_16;
          let r1 = s1 * c0 + s0 * c1 + carry;
          carry = r1 >>> 16;
          r1 &= m2_16;
          let r2 = s2 * c0 + s1 * c1 + s0 * c2 + carry;
          r2 &= m2_16;
          [s2, s1, s0] = [r2, r1, r0];
          return s2 * p2_16 + s1;
        };
        const next_signed = (bits) => {
          return _next() >> 32 - bits;
        };
        const next = (bits) => {
          return _next() >>> 32 - bits;
        };
        const checkIsNumber = (n) => {
          if (typeof n !== "number") {
            throw TypeError();
          }
        };
        const checkIsPositiveInt = (n, r = 2147483647) => {
          checkIsNumber(n);
          if (n < 0 || n > r) {
            throw RangeError();
          }
        };
        function setSeed(n) {
          checkIsNumber(n);
          s0 = n & m2_16 ^ c0;
          s1 = n / p2_16 & m2_16 ^ c1;
          s2 = n / p2_32 & m2_16 ^ c2;
        }
        function nextInt(bound) {
          if (bound === void 0) {
            return next_signed(32);
          }
          checkIsPositiveInt(bound);
          if ((bound & -bound) === bound) {
            let r = next(31) / p2_31;
            return ~~(bound * r);
          }
          var bits, val;
          do {
            bits = next(31);
            val = bits % bound;
          } while (bits - val + (bound - 1) < 0);
          return val;
        }
        function nextLong() {
          if (typeof BigInt !== "function") {
            throw Error("BigInt unsupported");
          }
          let msb = BigInt(next_signed(32));
          let lsb = BigInt(next_signed(32));
          const p2_32n = BigInt(p2_32);
          return msb * p2_32n + lsb;
        }
        function nextBoolean() {
          return next(1) != 0;
        }
        function nextFloat() {
          return next(24) / p2_24;
        }
        function nextDouble() {
          return (p2_27 * next(26) + next(27)) / p2_53;
        }
        function nextGaussian() {
          if (haveNextNextGaussian) {
            haveNextNextGaussian = false;
            return nextNextGaussian;
          } else {
            let v1, v2, s;
            do {
              v1 = 2 * nextDouble() - 1;
              v2 = 2 * nextDouble() - 1;
              s = v1 * v1 + v2 * v2;
            } while (s >= 1 || s === 0);
            let multiplier = Math.sqrt(-2 * Math.log(s) / s);
            nextNextGaussian = v2 * multiplier;
            haveNextNextGaussian = true;
            return v1 * multiplier;
          }
        }
        function checkStreamSize(streamSize) {
          if (streamSize === void 0) {
            return void 0;
          }
          checkIsPositiveInt(streamSize, Number.MAX_SAFE_INTEGER);
          return streamSize;
        }
        function* ints(streamSize) {
          streamSize = checkStreamSize(streamSize);
          let forever = streamSize === void 0;
          while (forever || streamSize-- > 0) {
            yield nextInt();
          }
        }
        function* longs(streamSize) {
          streamSize = checkStreamSize(streamSize);
          let forever = streamSize === void 0;
          while (forever || streamSize-- > 0) {
            yield nextLong();
          }
        }
        function* doubles(streamSize) {
          streamSize = checkStreamSize(streamSize);
          let forever = streamSize === void 0;
          while (forever || streamSize-- > 0) {
            yield nextDouble();
          }
        }
        const functions = {
          setSeed,
          nextInt,
          nextBoolean,
          nextFloat,
          nextDouble,
          nextGaussian,
          ints,
          doubles
        };
        if (typeof BigInt === "function") {
          Object.assign(functions, {
            nextLong,
            longs
          });
        }
        for (let f in functions) {
          functions[f] = { value: functions[f] };
        }
        Object.defineProperties(this, functions);
        if (seedval === void 0) {
          seedval = Math.floor(Math.random() * p2_48);
        }
        setSeed(seedval);
      }
    };
  }
});

// {TSPS_CHECKOUT}/client/rs/model/TextureMapper.ts
function computeTextureCoords(textureLoader, model) {
  const faceTextures = model.faceTextures;
  if (!faceTextures) {
    return void 0;
  }
  const verticesX = model.verticesX;
  const verticesY = model.verticesY;
  const verticesZ = model.verticesZ;
  const indices0 = model.indices1;
  const indices1 = model.indices2;
  const indices2 = model.indices3;
  const textureMappingP = model.textureMappingP;
  const textureMappingM = model.textureMappingM;
  const textureMappingN = model.textureMappingN;
  const faceCount = model.faceCount;
  const uvs = new Float32Array(faceCount * 6);
  const textureScales = calculateTextureScales(model);
  for (let i = 0; i < faceCount; i++) {
    let texCoord;
    if (model.textureCoords) {
      texCoord = model.textureCoords[i];
    } else {
      texCoord = -1;
    }
    let textureId = faceTextures[i];
    if (textureId !== -1 && !textureLoader.isSd(textureId)) {
      textureId = -1;
    }
    let u0 = 0;
    let v0 = 0;
    let u1 = 0;
    let v1 = 0;
    let u2 = 0;
    let v2 = 0;
    if (textureId !== -1) {
      let type = 0;
      if (texCoord !== -1) {
        texCoord &= 255;
        type = model.textureRenderTypes[texCoord];
      }
      const index0 = indices0[i];
      const index1 = indices1[i];
      const index2 = indices2[i];
      if (type === 0) {
        let p = index0;
        let m = index1;
        let n = index2;
        if (texCoord !== -1) {
          p = textureMappingP[texCoord];
          m = textureMappingM[texCoord];
          n = textureMappingN[texCoord];
        }
        const vx = verticesX[p];
        const vy = verticesY[p];
        const vz = verticesZ[p];
        const f_882_ = verticesX[m] - vx;
        const f_883_ = verticesY[m] - vy;
        const f_884_ = verticesZ[m] - vz;
        const f_885_ = verticesX[n] - vx;
        const f_886_ = verticesY[n] - vy;
        const f_887_ = verticesZ[n] - vz;
        const f_888_ = verticesX[index0] - vx;
        const f_889_ = verticesY[index0] - vy;
        const f_890_ = verticesZ[index0] - vz;
        const f_891_ = verticesX[index1] - vx;
        const f_892_ = verticesY[index1] - vy;
        const f_893_ = verticesZ[index1] - vz;
        const f_894_ = verticesX[index2] - vx;
        const f_895_ = verticesY[index2] - vy;
        const f_896_ = verticesZ[index2] - vz;
        const f_897_ = f_883_ * f_887_ - f_884_ * f_886_;
        const f_898_ = f_884_ * f_885_ - f_882_ * f_887_;
        const f_899_ = f_882_ * f_886_ - f_883_ * f_885_;
        let f_900_ = f_886_ * f_899_ - f_887_ * f_898_;
        let f_901_ = f_887_ * f_897_ - f_885_ * f_899_;
        let f_902_ = f_885_ * f_898_ - f_886_ * f_897_;
        let f_903_ = 1 / (f_900_ * f_882_ + f_901_ * f_883_ + f_902_ * f_884_);
        u0 = (f_900_ * f_888_ + f_901_ * f_889_ + f_902_ * f_890_) * f_903_;
        u1 = (f_900_ * f_891_ + f_901_ * f_892_ + f_902_ * f_893_) * f_903_;
        u2 = (f_900_ * f_894_ + f_901_ * f_895_ + f_902_ * f_896_) * f_903_;
        f_900_ = f_883_ * f_899_ - f_884_ * f_898_;
        f_901_ = f_884_ * f_897_ - f_882_ * f_899_;
        f_902_ = f_882_ * f_898_ - f_883_ * f_897_;
        f_903_ = 1 / (f_900_ * f_885_ + f_901_ * f_886_ + f_902_ * f_887_);
        v0 = (f_900_ * f_888_ + f_901_ * f_889_ + f_902_ * f_890_) * f_903_;
        v1 = (f_900_ * f_891_ + f_901_ * f_892_ + f_902_ * f_893_) * f_903_;
        v2 = (f_900_ * f_894_ + f_901_ * f_895_ + f_902_ * f_896_) * f_903_;
        if (u1 - u0 > 0.99 && u1 - u0 < 1.1) {
          u1 = 1;
        }
        if (u2 - u1 > 0.99 && u2 - u1 < 1.1) {
          u2 = 1;
        }
        if (u0 - u2 > 0.99 && u0 - u2 < 1.1) {
          u0 = 1;
        }
        if (u0 - u1 > 0.99 && u0 - u1 < 1.1) {
          u0 = 1;
        }
        if (u1 - u2 > 0.99 && u1 - u2 < 1.1) {
          u1 = 1;
        }
        if (u2 - u0 > 0.99 && u2 - u0 < 1.1) {
          u2 = 1;
        }
      } else if (textureScales.centerXs && textureScales.centerYs && textureScales.centerZs && textureScales.fs) {
        const centerX = textureScales.centerXs[texCoord];
        const centerY = textureScales.centerYs[texCoord];
        const centerZ = textureScales.centerZs[texCoord];
        const scales = textureScales.fs[texCoord];
        const direction = model.textureDirection[texCoord];
        const speed = model.textureSpeed[texCoord] / 256;
        if (type === 1) {
          const scaleZ = model.textureScaleZ[texCoord] / 1024;
          projectCylindrical(
            model.verticesX[index0],
            model.verticesY[index0],
            model.verticesZ[index0],
            centerX,
            centerY,
            centerZ,
            scales,
            scaleZ,
            direction,
            speed,
            uvTemp
          );
          u0 = uvTemp[0];
          v0 = uvTemp[1];
          projectCylindrical(
            model.verticesX[index1],
            model.verticesY[index1],
            model.verticesZ[index1],
            centerX,
            centerY,
            centerZ,
            scales,
            scaleZ,
            direction,
            speed,
            uvTemp
          );
          u1 = uvTemp[0];
          v1 = uvTemp[1];
          projectCylindrical(
            model.verticesX[index2],
            model.verticesY[index2],
            model.verticesZ[index2],
            centerX,
            centerY,
            centerZ,
            scales,
            scaleZ,
            direction,
            speed,
            uvTemp
          );
          u2 = uvTemp[0];
          v2 = uvTemp[1];
          const scaleZHalf = scaleZ / 2;
          if ((direction & 1) === 0) {
            if (u1 - u0 > scaleZHalf) {
              u1 -= scaleZ;
            } else if (u0 - u1 > scaleZHalf) {
              u1 += scaleZ;
            }
            if (u2 - u0 > scaleZHalf) {
              u2 -= scaleZ;
            } else if (u0 - u2 > scaleZHalf) {
              u2 += scaleZ;
            }
          } else {
            if (v1 - v0 > scaleZHalf) {
              v1 -= scaleZ;
            } else if (v0 - v1 > scaleZHalf) {
              v1 += scaleZ;
            }
            if (v2 - v0 > scaleZHalf) {
              v2 -= scaleZ;
            } else if (v0 - v2 > scaleZHalf) {
              v2 += scaleZ;
            }
          }
        } else if (type === 2) {
          const uOffset = model.textureTransU[texCoord] / 256;
          const vOffset = model.textureTransV[texCoord] / 256;
          const dx1 = model.verticesX[index1] - model.verticesX[index0];
          const dy1 = model.verticesY[index1] - model.verticesY[index0];
          const dz1 = model.verticesZ[index1] - model.verticesZ[index0];
          const dx2 = model.verticesX[index2] - model.verticesX[index0];
          const dy2 = model.verticesY[index2] - model.verticesY[index0];
          const dz2 = model.verticesZ[index2] - model.verticesZ[index0];
          const vx = dy1 * dz2 - dy2 * dz1;
          const vy = dz1 * dx2 - dz2 * dx1;
          const vz = dx1 * dy2 - dx2 * dy1;
          const scaleX = 64 / model.textureScaleX[texCoord];
          const scaleY = 64 / model.textureScaleY[texCoord];
          const scaleZ = 64 / model.textureScaleZ[texCoord];
          const f_829_ = (vx * scales[0] + vy * scales[1] + vz * scales[2]) / scaleX;
          const f_830_ = (vx * scales[3] + vy * scales[4] + vz * scales[5]) / scaleY;
          const f_831_ = (vx * scales[6] + vy * scales[7] + vz * scales[8]) / scaleZ;
          const scaleType = getDominantAxis(f_829_, f_830_, f_831_);
          projectPlanar(
            model.verticesX[index0],
            model.verticesY[index0],
            model.verticesZ[index0],
            centerX,
            centerY,
            centerZ,
            scaleType,
            scales,
            direction,
            speed,
            uOffset,
            vOffset,
            uvTemp
          );
          u0 = uvTemp[0];
          v0 = uvTemp[1];
          projectPlanar(
            model.verticesX[index1],
            model.verticesY[index1],
            model.verticesZ[index1],
            centerX,
            centerY,
            centerZ,
            scaleType,
            scales,
            direction,
            speed,
            uOffset,
            vOffset,
            uvTemp
          );
          u1 = uvTemp[0];
          v1 = uvTemp[1];
          projectPlanar(
            model.verticesX[index2],
            model.verticesY[index2],
            model.verticesZ[index2],
            centerX,
            centerY,
            centerZ,
            scaleType,
            scales,
            direction,
            speed,
            uOffset,
            vOffset,
            uvTemp
          );
          u2 = uvTemp[0];
          v2 = uvTemp[1];
        } else if (type === 3) {
          projectSpherical(
            model.verticesX[index0],
            model.verticesY[index0],
            model.verticesZ[index0],
            centerX,
            centerY,
            centerZ,
            scales,
            direction,
            speed,
            uvTemp
          );
          u0 = uvTemp[0];
          v0 = uvTemp[1];
          projectSpherical(
            model.verticesX[index1],
            model.verticesY[index1],
            model.verticesZ[index1],
            centerX,
            centerY,
            centerZ,
            scales,
            direction,
            speed,
            uvTemp
          );
          u1 = uvTemp[0];
          v1 = uvTemp[1];
          projectSpherical(
            model.verticesX[index2],
            model.verticesY[index2],
            model.verticesZ[index2],
            centerX,
            centerY,
            centerZ,
            scales,
            direction,
            speed,
            uvTemp
          );
          u2 = uvTemp[0];
          v2 = uvTemp[1];
          if ((direction & 1) == 0) {
            if (u1 - u0 > 0.5) {
              u1--;
            } else if (u0 - u1 > 0) {
              u1++;
            }
            if (u2 - u0 > 0.5) {
              u2--;
            } else if (u0 - u2 > 0.5) {
              u2++;
            }
          } else {
            if (v1 - v0 > 0.5) {
              v1--;
            } else if (v0 - v1 > 0.5) {
              v1++;
            }
            if (v2 - v0 > 0.5) {
              v2--;
            } else if (v0 - v2 > 0.5) {
              v2++;
            }
          }
        }
      }
    }
    const uvIndex = i * 6;
    uvs[uvIndex] = u0;
    uvs[uvIndex + 1] = v0;
    uvs[uvIndex + 2] = u1;
    uvs[uvIndex + 3] = v1;
    uvs[uvIndex + 4] = u2;
    uvs[uvIndex + 5] = v2;
  }
  return uvs;
}
function projectCylindrical(vx, vy, vz, centerX, centerY, centerZ, scales, scaleZ, direction, speed, out) {
  vx -= centerX;
  vy -= centerY;
  vz -= centerZ;
  const f_651_ = vx * scales[0] + vy * scales[1] + vz * scales[2];
  const f_652_ = vx * scales[3] + vy * scales[4] + vz * scales[5];
  const f_653_ = vx * scales[6] + vy * scales[7] + vz * scales[8];
  let u = Math.atan2(f_651_, f_653_) / 6.2831855 + 0.5;
  if (scaleZ !== 1) {
    u *= scaleZ;
  }
  let v = f_652_ + 0.5 + speed;
  if (direction === 1) {
    const f_656_ = u;
    u = -v;
    v = f_656_;
  } else if (direction === 2) {
    u = -u;
    v = -v;
  } else if (direction === 3) {
    const f_657_ = u;
    u = v;
    v = -f_657_;
  }
  out[0] = u;
  out[1] = v;
}
function getDominantAxis(f, f_715_, f_716_) {
  const f_717_ = f < 0 ? -f : f;
  const f_718_ = f_715_ < 0 ? -f_715_ : f_715_;
  const f_719_ = f_716_ < 0 ? -f_716_ : f_716_;
  if (f_718_ > f_717_ && f_718_ > f_719_) {
    if (f_715_ > 0) {
      return 0;
    }
    return 1;
  }
  if (f_719_ > f_717_ && f_719_ > f_718_) {
    if (f_716_ > 0) {
      return 2;
    }
    return 3;
  }
  if (f > 0) {
    return 4;
  }
  return 5;
}
function projectPlanar(vx, vy, vz, centerX, centerY, centerZ, scaleType, scales, direction, speed, uOffset, vOffset, out) {
  vx -= centerX;
  vy -= centerY;
  vz -= centerZ;
  const f_223_ = vx * scales[0] + vy * scales[1] + vz * scales[2];
  const f_224_ = vx * scales[3] + vy * scales[4] + vz * scales[5];
  const f_225_ = vx * scales[6] + vy * scales[7] + vz * scales[8];
  let u;
  let v;
  if (scaleType === 0) {
    u = f_223_ + speed + 0.5;
    v = -f_225_ + vOffset + 0.5;
  } else if (scaleType === 1) {
    u = f_223_ + speed + 0.5;
    v = f_225_ + vOffset + 0.5;
  } else if (scaleType === 2) {
    u = -f_223_ + speed + 0.5;
    v = -f_224_ + uOffset + 0.5;
  } else if (scaleType === 3) {
    u = f_223_ + speed + 0.5;
    v = -f_224_ + uOffset + 0.5;
  } else if (scaleType === 4) {
    u = f_225_ + vOffset + 0.5;
    v = -f_224_ + uOffset + 0.5;
  } else {
    u = -f_225_ + vOffset + 0.5;
    v = -f_224_ + uOffset + 0.5;
  }
  if (direction === 1) {
    const f_228_ = u;
    u = -v;
    v = f_228_;
  } else if (direction === 2) {
    u = -u;
    v = -v;
  } else if (direction === 3) {
    const f_229_ = u;
    u = v;
    v = -f_229_;
  }
  out[0] = u;
  out[1] = v;
}
function projectSpherical(vx, vy, vz, centerX, centerY, centerZ, scales, direction, speed, out) {
  vx -= centerX;
  vy -= centerY;
  vz -= centerZ;
  const f_682_ = vx * scales[0] + vy * scales[1] + vz * scales[2];
  const f_683_ = vx * scales[3] + vy * scales[4] + vz * scales[5];
  const f_684_ = vx * scales[6] + vy * scales[7] + vz * scales[8];
  const f_685_ = Math.sqrt(f_682_ * f_682_ + f_683_ * f_683_ + f_684_ * f_684_);
  let u = Math.atan2(f_682_, f_684_) / 6.2831855 + 0.5;
  let v = Math.asin(f_683_ / f_685_) / 3.1415927 + 0.5 + speed;
  if (direction === 1) {
    const f_688_ = u;
    u = -v;
    v = f_688_;
  } else if (direction === 2) {
    u = -u;
    v = -v;
  } else if (direction === 3) {
    const f_689_ = u;
    u = v;
    v = -f_689_;
  }
  out[0] = u;
  out[1] = v;
}
function calculateTextureScales(model) {
  let centerXs;
  let centerYs;
  let centerZs;
  let fs2;
  if (model.textureCoords) {
    const textureFaceCount = model.textureFaceCount;
    const minX = new Int32Array(textureFaceCount);
    const maxX = new Int32Array(textureFaceCount);
    const minY = new Int32Array(textureFaceCount);
    const maxY = new Int32Array(textureFaceCount);
    const minZ = new Int32Array(textureFaceCount);
    const maxZ = new Int32Array(textureFaceCount);
    for (let i = 0; i < textureFaceCount; i++) {
      minX[i] = 2147483647;
      maxX[i] = -2147483647;
      minY[i] = 2147483647;
      maxY[i] = -2147483647;
      minZ[i] = 2147483647;
      maxZ[i] = -2147483647;
    }
    fs2 = new Array(textureFaceCount);
    for (let i = 0; i < model.faceCount; i++) {
      if (model.textureCoords[i] === -1) {
        continue;
      }
      const texCoord = model.textureCoords[i] & 255;
      for (let v = 0; v < 3; v++) {
        let vertexIndex;
        if (v === 0) {
          vertexIndex = model.indices1[i];
        } else if (v === 1) {
          vertexIndex = model.indices2[i];
        } else {
          vertexIndex = model.indices3[i];
        }
        const vx = model.verticesX[vertexIndex];
        const vy = model.verticesY[vertexIndex];
        const vz = model.verticesZ[vertexIndex];
        if (minX[texCoord] > vx) {
          minX[texCoord] = vx;
        }
        if (vx > maxX[texCoord]) {
          maxX[texCoord] = vx;
        }
        if (minY[texCoord] > vy) {
          minY[texCoord] = vy;
        }
        if (maxY[texCoord] < vy) {
          maxY[texCoord] = vy;
        }
        if (minZ[texCoord] > vz) {
          minZ[texCoord] = vz;
        }
        if (maxZ[texCoord] < vz) {
          maxZ[texCoord] = vz;
        }
      }
    }
    centerXs = new Int32Array(textureFaceCount);
    centerYs = new Int32Array(textureFaceCount);
    centerZs = new Int32Array(textureFaceCount);
    for (let i = 0; i < textureFaceCount; i++) {
      const type = model.textureRenderTypes[i];
      if (type > 0) {
        centerXs[i] = (minX[i] + maxX[i]) / 2;
        centerYs[i] = (minY[i] + maxY[i]) / 2;
        centerZs[i] = (minZ[i] + maxZ[i]) / 2;
        let scaleX;
        let scaleY;
        let scaleZ;
        if (type === 1) {
          const scaleX0 = model.textureScaleX[i];
          scaleY = 64 / model.textureScaleY[i];
          if (scaleX0 === 0) {
            scaleZ = 1;
            scaleX = 1;
          } else if (scaleX0 <= 0) {
            scaleZ = 1;
            scaleX = -scaleX0 / 1024;
          } else {
            scaleX = 1;
            scaleZ = scaleX0 / 1024;
          }
        } else if (type === 2) {
          scaleX = 64 / model.textureScaleX[i];
          scaleY = 64 / model.textureScaleY[i];
          scaleZ = 64 / model.textureScaleZ[i];
        } else {
          scaleX = model.textureScaleX[i] / 1024;
          scaleY = model.textureScaleY[i] / 1024;
          scaleZ = model.textureScaleZ[i] / 1024;
        }
        fs2[i] = buildRotationScaleMatrix(
          model.textureMappingP[i],
          model.textureMappingM[i],
          model.textureMappingN[i],
          model.textureRotation[i] & 255,
          scaleX,
          scaleY,
          scaleZ
        );
      }
    }
  }
  return new TextureScales(centerXs, centerYs, centerZs, fs2);
}
function buildRotationScaleMatrix(p, m, n, rotation, scaleX, scaleY, scaleZ) {
  const fs2 = new Float32Array(9);
  let f_552_ = 1;
  let f_553_ = 0;
  let f_554_ = m / 32767;
  let f_555_ = -Math.sqrt(1 - f_554_ * f_554_);
  let f_556_ = 1 - f_554_;
  const f_557_ = Math.sqrt(p * p + n * n);
  if (f_557_ !== 0) {
    f_552_ = -n / f_557_;
    f_553_ = p / f_557_;
  }
  fs2[0] = f_554_ + f_552_ * f_552_ * f_556_;
  fs2[1] = f_553_ * f_555_;
  fs2[2] = f_553_ * f_552_ * f_556_;
  fs2[3] = -f_553_ * f_555_;
  fs2[4] = f_554_;
  fs2[5] = f_552_ * f_555_;
  fs2[6] = f_552_ * f_553_ * f_556_;
  fs2[7] = -f_552_ * f_555_;
  fs2[8] = f_554_ + f_553_ * f_553_ * f_556_;
  const fs_558_ = new Float32Array(9);
  f_554_ = Math.cos(rotation * 0.024543693);
  f_555_ = Math.sin(rotation * 0.024543693);
  f_556_ = 1 - f_554_;
  fs_558_[0] = f_554_;
  fs_558_[1] = 0;
  fs_558_[2] = f_555_;
  fs_558_[3] = 0;
  fs_558_[4] = 1;
  fs_558_[5] = 0;
  fs_558_[6] = -f_555_;
  fs_558_[7] = 0;
  fs_558_[8] = f_554_;
  const fs_559_ = new Float32Array(9);
  fs_559_[0] = fs_558_[0] * fs2[0] + fs_558_[1] * fs2[3] + fs_558_[2] * fs2[6];
  fs_559_[1] = fs_558_[0] * fs2[1] + fs_558_[1] * fs2[4] + fs_558_[2] * fs2[7];
  fs_559_[2] = fs_558_[0] * fs2[2] + fs_558_[1] * fs2[5] + fs_558_[2] * fs2[8];
  fs_559_[3] = fs_558_[3] * fs2[0] + fs_558_[4] * fs2[3] + fs_558_[5] * fs2[6];
  fs_559_[4] = fs_558_[3] * fs2[1] + fs_558_[4] * fs2[4] + fs_558_[5] * fs2[7];
  fs_559_[5] = fs_558_[3] * fs2[2] + fs_558_[4] * fs2[5] + fs_558_[5] * fs2[8];
  fs_559_[6] = fs_558_[6] * fs2[0] + fs_558_[7] * fs2[3] + fs_558_[8] * fs2[6];
  fs_559_[7] = fs_558_[6] * fs2[1] + fs_558_[7] * fs2[4] + fs_558_[8] * fs2[7];
  fs_559_[8] = fs_558_[6] * fs2[2] + fs_558_[7] * fs2[5] + fs_558_[8] * fs2[8];
  fs_559_[0] *= scaleX;
  fs_559_[1] *= scaleX;
  fs_559_[2] *= scaleX;
  fs_559_[3] *= scaleY;
  fs_559_[4] *= scaleY;
  fs_559_[5] *= scaleY;
  fs_559_[6] *= scaleZ;
  fs_559_[7] *= scaleZ;
  fs_559_[8] *= scaleZ;
  return fs_559_;
}
var uvTemp, TextureScales;
var init_TextureMapper = __esm({
  "{TSPS_CHECKOUT}/client/rs/model/TextureMapper.ts"() {
    "use strict";
    uvTemp = new Float32Array(2);
    TextureScales = class {
      constructor(centerXs, centerYs, centerZs, fs2) {
        this.centerXs = centerXs;
        this.centerYs = centerYs;
        this.centerZs = centerZs;
        this.fs = fs2;
      }
    };
  }
});

// {TSPS_CHECKOUT}/client/rs/MathConstants.ts
function initBitMasks() {
  const masks = new Int32Array(32);
  let value = 2;
  for (let i = 0; i < 32; i++) {
    masks[i] = value - 1;
    value += value;
  }
  return masks;
}
var TAU, RS_TO_RADIANS, RS_TO_DEGREES, DEGREES_TO_RADIANS, BIT_MASKS, SINE, COSINE, CIRCULAR_ANGLE, ANGULAR_RATIO, ANGULAR_RATIO_RADIANS, SINE_LARGE, COSINE_LARGE, d;
var init_MathConstants = __esm({
  "{TSPS_CHECKOUT}/client/rs/MathConstants.ts"() {
    "use strict";
    TAU = Math.PI * 2;
    RS_TO_RADIANS = TAU / 2048;
    RS_TO_DEGREES = RS_TO_RADIANS * 180 / Math.PI;
    DEGREES_TO_RADIANS = Math.PI / 180;
    BIT_MASKS = initBitMasks();
    SINE = new Int32Array(2048);
    COSINE = new Int32Array(2048);
    CIRCULAR_ANGLE = 2048;
    ANGULAR_RATIO = 360 / CIRCULAR_ANGLE;
    ANGULAR_RATIO_RADIANS = ANGULAR_RATIO * DEGREES_TO_RADIANS;
    for (let i = 0; i < 2048; i++) {
      SINE[i] = 65536 * Math.sin(i * ANGULAR_RATIO_RADIANS) | 0;
      COSINE[i] = 65536 * Math.cos(i * ANGULAR_RATIO_RADIANS) | 0;
    }
    SINE_LARGE = new Int32Array(16384);
    COSINE_LARGE = new Int32Array(16384);
    d = 3834951969714103e-19;
    for (let i = 0; i < 16384; i++) {
      SINE_LARGE[i] = 16384 * Math.sin(i * d);
      COSINE_LARGE[i] = 16384 * Math.cos(i * d);
    }
  }
});

// {TSPS_CHECKOUT}/client/rs/scene/entity/Entity.ts
var Entity;
var init_Entity = __esm({
  "{TSPS_CHECKOUT}/client/rs/scene/entity/Entity.ts"() {
    "use strict";
    Entity = class {
      constructor() {
        this.height = 1e3;
      }
      canMergeNormals() {
        return false;
      }
      mergeNormals(entity, offsetX, offsetY, offsetZ, hideOccluded) {
      }
      // light(textureLoader: TextureLoader, lightX: number, lightY: number, lightZ: number): Entity {
      //     return this;
      // }
    };
  }
});

// {TSPS_CHECKOUT}/client/rs/util/ColorUtil.ts
function buildPalette(brightness, var2, var3) {
  const palette = new Int32Array(65535);
  let paletteIndex = var2 * 128;
  for (let var5 = var2; var5 < var3; var5++) {
    const var6 = (var5 >> 3) / 64 + HUE_OFFSET;
    const var8 = (var5 & 7) / 8 + SATURATION_OFFSET;
    for (let var10 = 0; var10 < 128; var10++) {
      const var11 = var10 / 128;
      let var13 = var11;
      let var15 = var11;
      let var17 = var11;
      if (var8 !== 0) {
        let var19;
        if (var11 < 0.5) {
          var19 = var11 * (1 + var8);
        } else {
          var19 = var11 + var8 - var11 * var8;
        }
        const var21 = 2 * var11 - var19;
        let var23 = var6 + 0.3333333333333333;
        if (var23 > 1) {
          var23--;
        }
        let var27 = var6 - 0.3333333333333333;
        if (var27 < 0) {
          var27++;
        }
        if (6 * var23 < 1) {
          var13 = var21 + (var19 - var21) * 6 * var23;
        } else if (2 * var23 < 1) {
          var13 = var19;
        } else if (3 * var23 < 2) {
          var13 = var21 + (var19 - var21) * (0.6666666666666666 - var23) * 6;
        } else {
          var13 = var21;
        }
        if (6 * var6 < 1) {
          var15 = var21 + (var19 - var21) * 6 * var6;
        } else if (2 * var6 < 1) {
          var15 = var19;
        } else if (3 * var6 < 2) {
          var15 = var21 + (var19 - var21) * (0.6666666666666666 - var6) * 6;
        } else {
          var15 = var21;
        }
        if (6 * var27 < 1) {
          var17 = var21 + (var19 - var21) * 6 * var27;
        } else if (2 * var27 < 1) {
          var17 = var19;
        } else if (3 * var27 < 2) {
          var17 = var21 + (var19 - var21) * (0.6666666666666666 - var27) * 6;
        } else {
          var17 = var21;
        }
      }
      const r = var13 * 256 | 0;
      const g = var15 * 256 | 0;
      const b = var17 * 256 | 0;
      let rgb = (r << 16) + (g << 8) + b;
      rgb = brightenRgb(rgb, brightness);
      if (rgb === 0) {
        rgb = 1;
      }
      palette[paletteIndex++] = rgb;
    }
  }
  return palette;
}
function buildMinimapPalette() {
  const palette = new Int32Array(65536);
  const brightness = BRIGHTNESS_MINIMAP;
  for (let i = 0; i < 65536; i++) {
    const hue = HUE_OFFSET + (i >> 10 & 63) / 64;
    const saturation = SATURATION_OFFSET + (i >> 7 & 7) / 8;
    const luminance = (i & 127) / 128;
    let r = luminance;
    let g = luminance;
    let b = luminance;
    if (saturation !== 0) {
      let var15;
      if (luminance < 0.5) {
        var15 = (1 + saturation) * luminance;
      } else {
        var15 = luminance + saturation - saturation * luminance;
      }
      const var17 = 2 * luminance - var15;
      let var19 = 0.3333333333333333 + hue;
      if (var19 > 1) {
        var19--;
      }
      let var23 = hue - 0.3333333333333333;
      if (var23 < 0) {
        var23++;
      }
      if (var19 * 6 < 1) {
        r = var17 + 6 * (var15 - var17) * var19;
      } else if (2 * var19 < 1) {
        r = var15;
      } else if (var19 * 3 < 2) {
        r = (0.6666666666666666 - var19) * (var15 - var17) * 6 + var17;
      } else {
        r = var17;
      }
      if (hue * 6 < 1) {
        g = 6 * (var15 - var17) * hue + var17;
      } else if (hue * 2 < 1) {
        g = var15;
      } else if (hue * 3 < 2) {
        g = var17 + (var15 - var17) * (0.6666666666666666 - hue) * 6;
      } else {
        g = var17;
      }
      if (6 * var23 < 1) {
        b = var17 + 6 * (var15 - var17) * var23;
      } else if (2 * var23 < 1) {
        b = var15;
      } else if (3 * var23 < 2) {
        b = var17 + (var15 - var17) * (0.6666666666666666 - var23) * 6;
      } else {
        b = var17;
      }
    }
    r = Math.pow(r, brightness);
    g = Math.pow(g, brightness);
    b = Math.pow(b, brightness);
    const ri = 256 * r | 0;
    const gi = 256 * g | 0;
    const bi = b * 256 | 0;
    palette[i] = bi + (gi << 8) + (ri << 16) & 16777215;
  }
  return palette;
}
function brightenRgb(rgb, brightness) {
  let r = (rgb >> 16) / 256;
  let g = (rgb >> 8 & 255) / 256;
  let b = (rgb & 255) / 256;
  r = Math.pow(r, brightness);
  g = Math.pow(g, brightness);
  b = Math.pow(b, brightness);
  const newR = r * 256 | 0;
  const newG = g * 256 | 0;
  const newB = b * 256 | 0;
  return newR << 16 | newG << 8 | newB;
}
function blendLight(hsl, lightness) {
  lightness = (hsl & 127) * lightness >> 7;
  if (lightness < 2) {
    lightness = 2;
  } else if (lightness > 126) {
    lightness = 126;
  }
  return (hsl & 65408) + lightness;
}
function rgb15ToHsl(rgb15) {
  const r = (rgb15 >> 10 & 31) / 31;
  const g = (rgb15 >> 5 & 31) / 31;
  const b = (rgb15 & 31) / 31;
  let minRgb = r;
  if (g < r) {
    minRgb = g;
  }
  if (b < minRgb) {
    minRgb = b;
  }
  let maxRgb = r;
  if (g > r) {
    maxRgb = g;
  }
  if (b > maxRgb) {
    maxRgb = b;
  }
  let hueTemp = 0;
  let sat = 0;
  const light = (minRgb + maxRgb) / 2;
  if (minRgb !== maxRgb) {
    if (light < 0.5) {
      sat = (maxRgb - minRgb) / (minRgb + maxRgb);
    }
    if (light >= 0.5) {
      sat = (maxRgb - minRgb) / (2 - maxRgb - minRgb);
    }
    if (r === maxRgb) {
      hueTemp = (g - b) / (maxRgb - minRgb);
    } else if (maxRgb === g) {
      hueTemp = 2 + (b - r) / (maxRgb - minRgb);
    } else if (b === maxRgb) {
      hueTemp = 4 + (r - g) / (maxRgb - minRgb);
    }
  }
  let hue = (hueTemp * 256 / 6 | 0) & 255;
  let saturation = sat * 256;
  if (saturation < 0) {
    saturation = 0;
  } else if (saturation > 255) {
    saturation = 255;
  }
  if (light > 0.7) {
    saturation = Math.floor(saturation / 2);
  }
  if (light > 0.75) {
    saturation = Math.floor(saturation / 2);
  }
  if (light > 0.85) {
    saturation = Math.floor(saturation / 2);
  }
  if (light > 0.95) {
    saturation = Math.floor(saturation / 2);
  }
  let lightness = light;
  if (lightness > 0.995) {
    lightness = 0.995;
  }
  const packedHueSat = (hue / 4 | 0) * 8 + (saturation / 32 | 0);
  return (lightness * 128 | 0) + (packedHueSat << 7);
}
function buildRgb15ToHslMap() {
  const map = new Int32Array(32768);
  for (let i = 0; i < 32768; i++) {
    map[i] = rgb15ToHsl(i);
  }
  return map;
}
var BRIGHTNESS_LOW, BRIGHTNESS_MINIMAP, HUE_OFFSET, SATURATION_OFFSET, HSL_RGB_MAP, MINIMAP_HSL_RGB_MAP, RGB15_TO_HSL_MAP;
var init_ColorUtil = __esm({
  "{TSPS_CHECKOUT}/client/rs/util/ColorUtil.ts"() {
    "use strict";
    BRIGHTNESS_LOW = 0.8;
    BRIGHTNESS_MINIMAP = 0.949999988079071;
    HUE_OFFSET = 78125e-7;
    SATURATION_OFFSET = 0.0625;
    HSL_RGB_MAP = buildPalette(BRIGHTNESS_LOW, 0, 512);
    MINIMAP_HSL_RGB_MAP = buildMinimapPalette();
    RGB15_TO_HSL_MAP = buildRgb15ToHslMap();
  }
});

// {TSPS_CHECKOUT}/client/rs/model/FaceNormal.ts
var FaceNormal;
var init_FaceNormal = __esm({
  "{TSPS_CHECKOUT}/client/rs/model/FaceNormal.ts"() {
    "use strict";
    FaceNormal = class {
      constructor(x, y, z) {
        this.x = x;
        this.y = y;
        this.z = z;
      }
    };
  }
});

// {TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/common.js
var require_common = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/common.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.RANDOM = exports2.EPSILON = exports2.ARRAY_TYPE = exports2.ANGLE_ORDER = void 0;
    exports2.equals = equals;
    exports2.round = round;
    exports2.setMatrixArrayType = setMatrixArrayType;
    exports2.toDegree = toDegree;
    exports2.toRadian = toRadian;
    var EPSILON = exports2.EPSILON = 1e-6;
    var ARRAY_TYPE = exports2.ARRAY_TYPE = typeof Float32Array !== "undefined" ? Float32Array : Array;
    var RANDOM = exports2.RANDOM = Math.random;
    var ANGLE_ORDER = exports2.ANGLE_ORDER = "zyx";
    function round(a) {
      if (a >= 0) return Math.round(a);
      return a % 0.5 === 0 ? Math.floor(a) : Math.round(a);
    }
    function setMatrixArrayType(type) {
      exports2.ARRAY_TYPE = ARRAY_TYPE = type;
    }
    var degree = Math.PI / 180;
    var radian = 180 / Math.PI;
    function toRadian(a) {
      return a * degree;
    }
    function toDegree(a) {
      return a * radian;
    }
    function equals(a, b) {
      var tolerance = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : EPSILON;
      return Math.abs(a - b) <= tolerance * Math.max(1, Math.abs(a), Math.abs(b));
    }
  }
});

// {TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/mat2.js
var require_mat2 = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/mat2.js"(exports2) {
    "use strict";
    function _typeof(o) {
      "@babel/helpers - typeof";
      return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function(o2) {
        return typeof o2;
      } : function(o2) {
        return o2 && "function" == typeof Symbol && o2.constructor === Symbol && o2 !== Symbol.prototype ? "symbol" : typeof o2;
      }, _typeof(o);
    }
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.LDU = LDU;
    exports2.add = add;
    exports2.adjoint = adjoint;
    exports2.clone = clone;
    exports2.copy = copy;
    exports2.create = create;
    exports2.determinant = determinant;
    exports2.equals = equals;
    exports2.exactEquals = exactEquals;
    exports2.frob = frob;
    exports2.fromRotation = fromRotation;
    exports2.fromScaling = fromScaling;
    exports2.fromValues = fromValues;
    exports2.identity = identity;
    exports2.invert = invert;
    exports2.mul = void 0;
    exports2.multiply = multiply;
    exports2.multiplyScalar = multiplyScalar;
    exports2.multiplyScalarAndAdd = multiplyScalarAndAdd;
    exports2.rotate = rotate;
    exports2.scale = scale;
    exports2.set = set;
    exports2.str = str;
    exports2.sub = void 0;
    exports2.subtract = subtract;
    exports2.transpose = transpose;
    var glMatrix = _interopRequireWildcard(require_common());
    function _interopRequireWildcard(e, t) {
      if ("function" == typeof WeakMap) var r = /* @__PURE__ */ new WeakMap(), n = /* @__PURE__ */ new WeakMap();
      return (_interopRequireWildcard = function _interopRequireWildcard2(e2, t2) {
        if (!t2 && e2 && e2.__esModule) return e2;
        var o, i, f = { __proto__: null, "default": e2 };
        if (null === e2 || "object" != _typeof(e2) && "function" != typeof e2) return f;
        if (o = t2 ? n : r) {
          if (o.has(e2)) return o.get(e2);
          o.set(e2, f);
        }
        for (var _t in e2) "default" !== _t && {}.hasOwnProperty.call(e2, _t) && ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e2, _t)) && (i.get || i.set) ? o(f, _t, i) : f[_t] = e2[_t]);
        return f;
      })(e, t);
    }
    function create() {
      var out = new glMatrix.ARRAY_TYPE(4);
      if (glMatrix.ARRAY_TYPE != Float32Array) {
        out[1] = 0;
        out[2] = 0;
      }
      out[0] = 1;
      out[3] = 1;
      return out;
    }
    function clone(a) {
      var out = new glMatrix.ARRAY_TYPE(4);
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      out[3] = a[3];
      return out;
    }
    function copy(out, a) {
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      out[3] = a[3];
      return out;
    }
    function identity(out) {
      out[0] = 1;
      out[1] = 0;
      out[2] = 0;
      out[3] = 1;
      return out;
    }
    function fromValues(m00, m01, m10, m11) {
      var out = new glMatrix.ARRAY_TYPE(4);
      out[0] = m00;
      out[1] = m01;
      out[2] = m10;
      out[3] = m11;
      return out;
    }
    function set(out, m00, m01, m10, m11) {
      out[0] = m00;
      out[1] = m01;
      out[2] = m10;
      out[3] = m11;
      return out;
    }
    function transpose(out, a) {
      if (out === a) {
        var a1 = a[1];
        out[1] = a[2];
        out[2] = a1;
      } else {
        out[0] = a[0];
        out[1] = a[2];
        out[2] = a[1];
        out[3] = a[3];
      }
      return out;
    }
    function invert(out, a) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3];
      var det = a0 * a3 - a2 * a1;
      if (!det) {
        return null;
      }
      det = 1 / det;
      out[0] = a3 * det;
      out[1] = -a1 * det;
      out[2] = -a2 * det;
      out[3] = a0 * det;
      return out;
    }
    function adjoint(out, a) {
      var a0 = a[0];
      out[0] = a[3];
      out[1] = -a[1];
      out[2] = -a[2];
      out[3] = a0;
      return out;
    }
    function determinant(a) {
      return a[0] * a[3] - a[2] * a[1];
    }
    function multiply(out, a, b) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3];
      var b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3];
      out[0] = a0 * b0 + a2 * b1;
      out[1] = a1 * b0 + a3 * b1;
      out[2] = a0 * b2 + a2 * b3;
      out[3] = a1 * b2 + a3 * b3;
      return out;
    }
    function rotate(out, a, rad) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3];
      var s = Math.sin(rad);
      var c = Math.cos(rad);
      out[0] = a0 * c + a2 * s;
      out[1] = a1 * c + a3 * s;
      out[2] = a0 * -s + a2 * c;
      out[3] = a1 * -s + a3 * c;
      return out;
    }
    function scale(out, a, v) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3];
      var v0 = v[0], v1 = v[1];
      out[0] = a0 * v0;
      out[1] = a1 * v0;
      out[2] = a2 * v1;
      out[3] = a3 * v1;
      return out;
    }
    function fromRotation(out, rad) {
      var s = Math.sin(rad);
      var c = Math.cos(rad);
      out[0] = c;
      out[1] = s;
      out[2] = -s;
      out[3] = c;
      return out;
    }
    function fromScaling(out, v) {
      out[0] = v[0];
      out[1] = 0;
      out[2] = 0;
      out[3] = v[1];
      return out;
    }
    function str(a) {
      return "mat2(" + a[0] + ", " + a[1] + ", " + a[2] + ", " + a[3] + ")";
    }
    function frob(a) {
      return Math.sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2] + a[3] * a[3]);
    }
    function LDU(L, D, U, a) {
      L[2] = a[2] / a[0];
      U[0] = a[0];
      U[1] = a[1];
      U[3] = a[3] - L[2] * U[1];
      return [L, D, U];
    }
    function add(out, a, b) {
      out[0] = a[0] + b[0];
      out[1] = a[1] + b[1];
      out[2] = a[2] + b[2];
      out[3] = a[3] + b[3];
      return out;
    }
    function subtract(out, a, b) {
      out[0] = a[0] - b[0];
      out[1] = a[1] - b[1];
      out[2] = a[2] - b[2];
      out[3] = a[3] - b[3];
      return out;
    }
    function exactEquals(a, b) {
      return a[0] === b[0] && a[1] === b[1] && a[2] === b[2] && a[3] === b[3];
    }
    function equals(a, b) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3];
      var b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3];
      return Math.abs(a0 - b0) <= glMatrix.EPSILON * Math.max(1, Math.abs(a0), Math.abs(b0)) && Math.abs(a1 - b1) <= glMatrix.EPSILON * Math.max(1, Math.abs(a1), Math.abs(b1)) && Math.abs(a2 - b2) <= glMatrix.EPSILON * Math.max(1, Math.abs(a2), Math.abs(b2)) && Math.abs(a3 - b3) <= glMatrix.EPSILON * Math.max(1, Math.abs(a3), Math.abs(b3));
    }
    function multiplyScalar(out, a, b) {
      out[0] = a[0] * b;
      out[1] = a[1] * b;
      out[2] = a[2] * b;
      out[3] = a[3] * b;
      return out;
    }
    function multiplyScalarAndAdd(out, a, b, scale2) {
      out[0] = a[0] + b[0] * scale2;
      out[1] = a[1] + b[1] * scale2;
      out[2] = a[2] + b[2] * scale2;
      out[3] = a[3] + b[3] * scale2;
      return out;
    }
    var mul = exports2.mul = multiply;
    var sub = exports2.sub = subtract;
  }
});

// {TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/mat2d.js
var require_mat2d = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/mat2d.js"(exports2) {
    "use strict";
    function _typeof(o) {
      "@babel/helpers - typeof";
      return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function(o2) {
        return typeof o2;
      } : function(o2) {
        return o2 && "function" == typeof Symbol && o2.constructor === Symbol && o2 !== Symbol.prototype ? "symbol" : typeof o2;
      }, _typeof(o);
    }
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.add = add;
    exports2.clone = clone;
    exports2.copy = copy;
    exports2.create = create;
    exports2.determinant = determinant;
    exports2.equals = equals;
    exports2.exactEquals = exactEquals;
    exports2.frob = frob;
    exports2.fromRotation = fromRotation;
    exports2.fromScaling = fromScaling;
    exports2.fromTranslation = fromTranslation;
    exports2.fromValues = fromValues;
    exports2.identity = identity;
    exports2.invert = invert;
    exports2.mul = void 0;
    exports2.multiply = multiply;
    exports2.multiplyScalar = multiplyScalar;
    exports2.multiplyScalarAndAdd = multiplyScalarAndAdd;
    exports2.rotate = rotate;
    exports2.scale = scale;
    exports2.set = set;
    exports2.str = str;
    exports2.sub = void 0;
    exports2.subtract = subtract;
    exports2.translate = translate;
    var glMatrix = _interopRequireWildcard(require_common());
    function _interopRequireWildcard(e, t) {
      if ("function" == typeof WeakMap) var r = /* @__PURE__ */ new WeakMap(), n = /* @__PURE__ */ new WeakMap();
      return (_interopRequireWildcard = function _interopRequireWildcard2(e2, t2) {
        if (!t2 && e2 && e2.__esModule) return e2;
        var o, i, f = { __proto__: null, "default": e2 };
        if (null === e2 || "object" != _typeof(e2) && "function" != typeof e2) return f;
        if (o = t2 ? n : r) {
          if (o.has(e2)) return o.get(e2);
          o.set(e2, f);
        }
        for (var _t in e2) "default" !== _t && {}.hasOwnProperty.call(e2, _t) && ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e2, _t)) && (i.get || i.set) ? o(f, _t, i) : f[_t] = e2[_t]);
        return f;
      })(e, t);
    }
    function create() {
      var out = new glMatrix.ARRAY_TYPE(6);
      if (glMatrix.ARRAY_TYPE != Float32Array) {
        out[1] = 0;
        out[2] = 0;
        out[4] = 0;
        out[5] = 0;
      }
      out[0] = 1;
      out[3] = 1;
      return out;
    }
    function clone(a) {
      var out = new glMatrix.ARRAY_TYPE(6);
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      out[3] = a[3];
      out[4] = a[4];
      out[5] = a[5];
      return out;
    }
    function copy(out, a) {
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      out[3] = a[3];
      out[4] = a[4];
      out[5] = a[5];
      return out;
    }
    function identity(out) {
      out[0] = 1;
      out[1] = 0;
      out[2] = 0;
      out[3] = 1;
      out[4] = 0;
      out[5] = 0;
      return out;
    }
    function fromValues(a, b, c, d2, tx, ty) {
      var out = new glMatrix.ARRAY_TYPE(6);
      out[0] = a;
      out[1] = b;
      out[2] = c;
      out[3] = d2;
      out[4] = tx;
      out[5] = ty;
      return out;
    }
    function set(out, a, b, c, d2, tx, ty) {
      out[0] = a;
      out[1] = b;
      out[2] = c;
      out[3] = d2;
      out[4] = tx;
      out[5] = ty;
      return out;
    }
    function invert(out, a) {
      var aa = a[0], ab = a[1], ac = a[2], ad = a[3];
      var atx = a[4], aty = a[5];
      var det = aa * ad - ab * ac;
      if (!det) {
        return null;
      }
      det = 1 / det;
      out[0] = ad * det;
      out[1] = -ab * det;
      out[2] = -ac * det;
      out[3] = aa * det;
      out[4] = (ac * aty - ad * atx) * det;
      out[5] = (ab * atx - aa * aty) * det;
      return out;
    }
    function determinant(a) {
      return a[0] * a[3] - a[1] * a[2];
    }
    function multiply(out, a, b) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3], a4 = a[4], a5 = a[5];
      var b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3], b4 = b[4], b5 = b[5];
      out[0] = a0 * b0 + a2 * b1;
      out[1] = a1 * b0 + a3 * b1;
      out[2] = a0 * b2 + a2 * b3;
      out[3] = a1 * b2 + a3 * b3;
      out[4] = a0 * b4 + a2 * b5 + a4;
      out[5] = a1 * b4 + a3 * b5 + a5;
      return out;
    }
    function rotate(out, a, rad) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3], a4 = a[4], a5 = a[5];
      var s = Math.sin(rad);
      var c = Math.cos(rad);
      out[0] = a0 * c + a2 * s;
      out[1] = a1 * c + a3 * s;
      out[2] = a0 * -s + a2 * c;
      out[3] = a1 * -s + a3 * c;
      out[4] = a4;
      out[5] = a5;
      return out;
    }
    function scale(out, a, v) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3], a4 = a[4], a5 = a[5];
      var v0 = v[0], v1 = v[1];
      out[0] = a0 * v0;
      out[1] = a1 * v0;
      out[2] = a2 * v1;
      out[3] = a3 * v1;
      out[4] = a4;
      out[5] = a5;
      return out;
    }
    function translate(out, a, v) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3], a4 = a[4], a5 = a[5];
      var v0 = v[0], v1 = v[1];
      out[0] = a0;
      out[1] = a1;
      out[2] = a2;
      out[3] = a3;
      out[4] = a0 * v0 + a2 * v1 + a4;
      out[5] = a1 * v0 + a3 * v1 + a5;
      return out;
    }
    function fromRotation(out, rad) {
      var s = Math.sin(rad), c = Math.cos(rad);
      out[0] = c;
      out[1] = s;
      out[2] = -s;
      out[3] = c;
      out[4] = 0;
      out[5] = 0;
      return out;
    }
    function fromScaling(out, v) {
      out[0] = v[0];
      out[1] = 0;
      out[2] = 0;
      out[3] = v[1];
      out[4] = 0;
      out[5] = 0;
      return out;
    }
    function fromTranslation(out, v) {
      out[0] = 1;
      out[1] = 0;
      out[2] = 0;
      out[3] = 1;
      out[4] = v[0];
      out[5] = v[1];
      return out;
    }
    function str(a) {
      return "mat2d(" + a[0] + ", " + a[1] + ", " + a[2] + ", " + a[3] + ", " + a[4] + ", " + a[5] + ")";
    }
    function frob(a) {
      return Math.sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2] + a[3] * a[3] + a[4] * a[4] + a[5] * a[5] + 1);
    }
    function add(out, a, b) {
      out[0] = a[0] + b[0];
      out[1] = a[1] + b[1];
      out[2] = a[2] + b[2];
      out[3] = a[3] + b[3];
      out[4] = a[4] + b[4];
      out[5] = a[5] + b[5];
      return out;
    }
    function subtract(out, a, b) {
      out[0] = a[0] - b[0];
      out[1] = a[1] - b[1];
      out[2] = a[2] - b[2];
      out[3] = a[3] - b[3];
      out[4] = a[4] - b[4];
      out[5] = a[5] - b[5];
      return out;
    }
    function multiplyScalar(out, a, b) {
      out[0] = a[0] * b;
      out[1] = a[1] * b;
      out[2] = a[2] * b;
      out[3] = a[3] * b;
      out[4] = a[4] * b;
      out[5] = a[5] * b;
      return out;
    }
    function multiplyScalarAndAdd(out, a, b, scale2) {
      out[0] = a[0] + b[0] * scale2;
      out[1] = a[1] + b[1] * scale2;
      out[2] = a[2] + b[2] * scale2;
      out[3] = a[3] + b[3] * scale2;
      out[4] = a[4] + b[4] * scale2;
      out[5] = a[5] + b[5] * scale2;
      return out;
    }
    function exactEquals(a, b) {
      return a[0] === b[0] && a[1] === b[1] && a[2] === b[2] && a[3] === b[3] && a[4] === b[4] && a[5] === b[5];
    }
    function equals(a, b) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3], a4 = a[4], a5 = a[5];
      var b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3], b4 = b[4], b5 = b[5];
      return Math.abs(a0 - b0) <= glMatrix.EPSILON * Math.max(1, Math.abs(a0), Math.abs(b0)) && Math.abs(a1 - b1) <= glMatrix.EPSILON * Math.max(1, Math.abs(a1), Math.abs(b1)) && Math.abs(a2 - b2) <= glMatrix.EPSILON * Math.max(1, Math.abs(a2), Math.abs(b2)) && Math.abs(a3 - b3) <= glMatrix.EPSILON * Math.max(1, Math.abs(a3), Math.abs(b3)) && Math.abs(a4 - b4) <= glMatrix.EPSILON * Math.max(1, Math.abs(a4), Math.abs(b4)) && Math.abs(a5 - b5) <= glMatrix.EPSILON * Math.max(1, Math.abs(a5), Math.abs(b5));
    }
    var mul = exports2.mul = multiply;
    var sub = exports2.sub = subtract;
  }
});

// {TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/mat3.js
var require_mat3 = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/mat3.js"(exports2) {
    "use strict";
    function _typeof(o) {
      "@babel/helpers - typeof";
      return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function(o2) {
        return typeof o2;
      } : function(o2) {
        return o2 && "function" == typeof Symbol && o2.constructor === Symbol && o2 !== Symbol.prototype ? "symbol" : typeof o2;
      }, _typeof(o);
    }
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.add = add;
    exports2.adjoint = adjoint;
    exports2.clone = clone;
    exports2.copy = copy;
    exports2.create = create;
    exports2.determinant = determinant;
    exports2.equals = equals;
    exports2.exactEquals = exactEquals;
    exports2.frob = frob;
    exports2.fromMat2d = fromMat2d;
    exports2.fromMat4 = fromMat4;
    exports2.fromQuat = fromQuat;
    exports2.fromRotation = fromRotation;
    exports2.fromScaling = fromScaling;
    exports2.fromTranslation = fromTranslation;
    exports2.fromValues = fromValues;
    exports2.identity = identity;
    exports2.invert = invert;
    exports2.mul = void 0;
    exports2.multiply = multiply;
    exports2.multiplyScalar = multiplyScalar;
    exports2.multiplyScalarAndAdd = multiplyScalarAndAdd;
    exports2.normalFromMat4 = normalFromMat4;
    exports2.projection = projection;
    exports2.rotate = rotate;
    exports2.scale = scale;
    exports2.set = set;
    exports2.str = str;
    exports2.sub = void 0;
    exports2.subtract = subtract;
    exports2.translate = translate;
    exports2.transpose = transpose;
    var glMatrix = _interopRequireWildcard(require_common());
    function _interopRequireWildcard(e, t) {
      if ("function" == typeof WeakMap) var r = /* @__PURE__ */ new WeakMap(), n = /* @__PURE__ */ new WeakMap();
      return (_interopRequireWildcard = function _interopRequireWildcard2(e2, t2) {
        if (!t2 && e2 && e2.__esModule) return e2;
        var o, i, f = { __proto__: null, "default": e2 };
        if (null === e2 || "object" != _typeof(e2) && "function" != typeof e2) return f;
        if (o = t2 ? n : r) {
          if (o.has(e2)) return o.get(e2);
          o.set(e2, f);
        }
        for (var _t in e2) "default" !== _t && {}.hasOwnProperty.call(e2, _t) && ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e2, _t)) && (i.get || i.set) ? o(f, _t, i) : f[_t] = e2[_t]);
        return f;
      })(e, t);
    }
    function create() {
      var out = new glMatrix.ARRAY_TYPE(9);
      if (glMatrix.ARRAY_TYPE != Float32Array) {
        out[1] = 0;
        out[2] = 0;
        out[3] = 0;
        out[5] = 0;
        out[6] = 0;
        out[7] = 0;
      }
      out[0] = 1;
      out[4] = 1;
      out[8] = 1;
      return out;
    }
    function fromMat4(out, a) {
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      out[3] = a[4];
      out[4] = a[5];
      out[5] = a[6];
      out[6] = a[8];
      out[7] = a[9];
      out[8] = a[10];
      return out;
    }
    function clone(a) {
      var out = new glMatrix.ARRAY_TYPE(9);
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      out[3] = a[3];
      out[4] = a[4];
      out[5] = a[5];
      out[6] = a[6];
      out[7] = a[7];
      out[8] = a[8];
      return out;
    }
    function copy(out, a) {
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      out[3] = a[3];
      out[4] = a[4];
      out[5] = a[5];
      out[6] = a[6];
      out[7] = a[7];
      out[8] = a[8];
      return out;
    }
    function fromValues(m00, m01, m02, m10, m11, m12, m20, m21, m22) {
      var out = new glMatrix.ARRAY_TYPE(9);
      out[0] = m00;
      out[1] = m01;
      out[2] = m02;
      out[3] = m10;
      out[4] = m11;
      out[5] = m12;
      out[6] = m20;
      out[7] = m21;
      out[8] = m22;
      return out;
    }
    function set(out, m00, m01, m02, m10, m11, m12, m20, m21, m22) {
      out[0] = m00;
      out[1] = m01;
      out[2] = m02;
      out[3] = m10;
      out[4] = m11;
      out[5] = m12;
      out[6] = m20;
      out[7] = m21;
      out[8] = m22;
      return out;
    }
    function identity(out) {
      out[0] = 1;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 1;
      out[5] = 0;
      out[6] = 0;
      out[7] = 0;
      out[8] = 1;
      return out;
    }
    function transpose(out, a) {
      if (out === a) {
        var a01 = a[1], a02 = a[2], a12 = a[5];
        out[1] = a[3];
        out[2] = a[6];
        out[3] = a01;
        out[5] = a[7];
        out[6] = a02;
        out[7] = a12;
      } else {
        out[0] = a[0];
        out[1] = a[3];
        out[2] = a[6];
        out[3] = a[1];
        out[4] = a[4];
        out[5] = a[7];
        out[6] = a[2];
        out[7] = a[5];
        out[8] = a[8];
      }
      return out;
    }
    function invert(out, a) {
      var a00 = a[0], a01 = a[1], a02 = a[2];
      var a10 = a[3], a11 = a[4], a12 = a[5];
      var a20 = a[6], a21 = a[7], a22 = a[8];
      var b01 = a22 * a11 - a12 * a21;
      var b11 = -a22 * a10 + a12 * a20;
      var b21 = a21 * a10 - a11 * a20;
      var det = a00 * b01 + a01 * b11 + a02 * b21;
      if (!det) {
        return null;
      }
      det = 1 / det;
      out[0] = b01 * det;
      out[1] = (-a22 * a01 + a02 * a21) * det;
      out[2] = (a12 * a01 - a02 * a11) * det;
      out[3] = b11 * det;
      out[4] = (a22 * a00 - a02 * a20) * det;
      out[5] = (-a12 * a00 + a02 * a10) * det;
      out[6] = b21 * det;
      out[7] = (-a21 * a00 + a01 * a20) * det;
      out[8] = (a11 * a00 - a01 * a10) * det;
      return out;
    }
    function adjoint(out, a) {
      var a00 = a[0], a01 = a[1], a02 = a[2];
      var a10 = a[3], a11 = a[4], a12 = a[5];
      var a20 = a[6], a21 = a[7], a22 = a[8];
      out[0] = a11 * a22 - a12 * a21;
      out[1] = a02 * a21 - a01 * a22;
      out[2] = a01 * a12 - a02 * a11;
      out[3] = a12 * a20 - a10 * a22;
      out[4] = a00 * a22 - a02 * a20;
      out[5] = a02 * a10 - a00 * a12;
      out[6] = a10 * a21 - a11 * a20;
      out[7] = a01 * a20 - a00 * a21;
      out[8] = a00 * a11 - a01 * a10;
      return out;
    }
    function determinant(a) {
      var a00 = a[0], a01 = a[1], a02 = a[2];
      var a10 = a[3], a11 = a[4], a12 = a[5];
      var a20 = a[6], a21 = a[7], a22 = a[8];
      return a00 * (a22 * a11 - a12 * a21) + a01 * (-a22 * a10 + a12 * a20) + a02 * (a21 * a10 - a11 * a20);
    }
    function multiply(out, a, b) {
      var a00 = a[0], a01 = a[1], a02 = a[2];
      var a10 = a[3], a11 = a[4], a12 = a[5];
      var a20 = a[6], a21 = a[7], a22 = a[8];
      var b00 = b[0], b01 = b[1], b02 = b[2];
      var b10 = b[3], b11 = b[4], b12 = b[5];
      var b20 = b[6], b21 = b[7], b22 = b[8];
      out[0] = b00 * a00 + b01 * a10 + b02 * a20;
      out[1] = b00 * a01 + b01 * a11 + b02 * a21;
      out[2] = b00 * a02 + b01 * a12 + b02 * a22;
      out[3] = b10 * a00 + b11 * a10 + b12 * a20;
      out[4] = b10 * a01 + b11 * a11 + b12 * a21;
      out[5] = b10 * a02 + b11 * a12 + b12 * a22;
      out[6] = b20 * a00 + b21 * a10 + b22 * a20;
      out[7] = b20 * a01 + b21 * a11 + b22 * a21;
      out[8] = b20 * a02 + b21 * a12 + b22 * a22;
      return out;
    }
    function translate(out, a, v) {
      var a00 = a[0], a01 = a[1], a02 = a[2], a10 = a[3], a11 = a[4], a12 = a[5], a20 = a[6], a21 = a[7], a22 = a[8], x = v[0], y = v[1];
      out[0] = a00;
      out[1] = a01;
      out[2] = a02;
      out[3] = a10;
      out[4] = a11;
      out[5] = a12;
      out[6] = x * a00 + y * a10 + a20;
      out[7] = x * a01 + y * a11 + a21;
      out[8] = x * a02 + y * a12 + a22;
      return out;
    }
    function rotate(out, a, rad) {
      var a00 = a[0], a01 = a[1], a02 = a[2], a10 = a[3], a11 = a[4], a12 = a[5], a20 = a[6], a21 = a[7], a22 = a[8], s = Math.sin(rad), c = Math.cos(rad);
      out[0] = c * a00 + s * a10;
      out[1] = c * a01 + s * a11;
      out[2] = c * a02 + s * a12;
      out[3] = c * a10 - s * a00;
      out[4] = c * a11 - s * a01;
      out[5] = c * a12 - s * a02;
      out[6] = a20;
      out[7] = a21;
      out[8] = a22;
      return out;
    }
    function scale(out, a, v) {
      var x = v[0], y = v[1];
      out[0] = x * a[0];
      out[1] = x * a[1];
      out[2] = x * a[2];
      out[3] = y * a[3];
      out[4] = y * a[4];
      out[5] = y * a[5];
      out[6] = a[6];
      out[7] = a[7];
      out[8] = a[8];
      return out;
    }
    function fromTranslation(out, v) {
      out[0] = 1;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 1;
      out[5] = 0;
      out[6] = v[0];
      out[7] = v[1];
      out[8] = 1;
      return out;
    }
    function fromRotation(out, rad) {
      var s = Math.sin(rad), c = Math.cos(rad);
      out[0] = c;
      out[1] = s;
      out[2] = 0;
      out[3] = -s;
      out[4] = c;
      out[5] = 0;
      out[6] = 0;
      out[7] = 0;
      out[8] = 1;
      return out;
    }
    function fromScaling(out, v) {
      out[0] = v[0];
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = v[1];
      out[5] = 0;
      out[6] = 0;
      out[7] = 0;
      out[8] = 1;
      return out;
    }
    function fromMat2d(out, a) {
      out[0] = a[0];
      out[1] = a[1];
      out[2] = 0;
      out[3] = a[2];
      out[4] = a[3];
      out[5] = 0;
      out[6] = a[4];
      out[7] = a[5];
      out[8] = 1;
      return out;
    }
    function fromQuat(out, q) {
      var x = q[0], y = q[1], z = q[2], w = q[3];
      var x2 = x + x;
      var y2 = y + y;
      var z2 = z + z;
      var xx = x * x2;
      var yx = y * x2;
      var yy = y * y2;
      var zx = z * x2;
      var zy = z * y2;
      var zz = z * z2;
      var wx = w * x2;
      var wy = w * y2;
      var wz = w * z2;
      out[0] = 1 - yy - zz;
      out[3] = yx - wz;
      out[6] = zx + wy;
      out[1] = yx + wz;
      out[4] = 1 - xx - zz;
      out[7] = zy - wx;
      out[2] = zx - wy;
      out[5] = zy + wx;
      out[8] = 1 - xx - yy;
      return out;
    }
    function normalFromMat4(out, a) {
      var a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
      var a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
      var a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
      var a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];
      var b00 = a00 * a11 - a01 * a10;
      var b01 = a00 * a12 - a02 * a10;
      var b02 = a00 * a13 - a03 * a10;
      var b03 = a01 * a12 - a02 * a11;
      var b04 = a01 * a13 - a03 * a11;
      var b05 = a02 * a13 - a03 * a12;
      var b06 = a20 * a31 - a21 * a30;
      var b07 = a20 * a32 - a22 * a30;
      var b08 = a20 * a33 - a23 * a30;
      var b09 = a21 * a32 - a22 * a31;
      var b10 = a21 * a33 - a23 * a31;
      var b11 = a22 * a33 - a23 * a32;
      var det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
      if (!det) {
        return null;
      }
      det = 1 / det;
      out[0] = (a11 * b11 - a12 * b10 + a13 * b09) * det;
      out[1] = (a12 * b08 - a10 * b11 - a13 * b07) * det;
      out[2] = (a10 * b10 - a11 * b08 + a13 * b06) * det;
      out[3] = (a02 * b10 - a01 * b11 - a03 * b09) * det;
      out[4] = (a00 * b11 - a02 * b08 + a03 * b07) * det;
      out[5] = (a01 * b08 - a00 * b10 - a03 * b06) * det;
      out[6] = (a31 * b05 - a32 * b04 + a33 * b03) * det;
      out[7] = (a32 * b02 - a30 * b05 - a33 * b01) * det;
      out[8] = (a30 * b04 - a31 * b02 + a33 * b00) * det;
      return out;
    }
    function projection(out, width, height) {
      out[0] = 2 / width;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = -2 / height;
      out[5] = 0;
      out[6] = -1;
      out[7] = 1;
      out[8] = 1;
      return out;
    }
    function str(a) {
      return "mat3(" + a[0] + ", " + a[1] + ", " + a[2] + ", " + a[3] + ", " + a[4] + ", " + a[5] + ", " + a[6] + ", " + a[7] + ", " + a[8] + ")";
    }
    function frob(a) {
      return Math.sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2] + a[3] * a[3] + a[4] * a[4] + a[5] * a[5] + a[6] * a[6] + a[7] * a[7] + a[8] * a[8]);
    }
    function add(out, a, b) {
      out[0] = a[0] + b[0];
      out[1] = a[1] + b[1];
      out[2] = a[2] + b[2];
      out[3] = a[3] + b[3];
      out[4] = a[4] + b[4];
      out[5] = a[5] + b[5];
      out[6] = a[6] + b[6];
      out[7] = a[7] + b[7];
      out[8] = a[8] + b[8];
      return out;
    }
    function subtract(out, a, b) {
      out[0] = a[0] - b[0];
      out[1] = a[1] - b[1];
      out[2] = a[2] - b[2];
      out[3] = a[3] - b[3];
      out[4] = a[4] - b[4];
      out[5] = a[5] - b[5];
      out[6] = a[6] - b[6];
      out[7] = a[7] - b[7];
      out[8] = a[8] - b[8];
      return out;
    }
    function multiplyScalar(out, a, b) {
      out[0] = a[0] * b;
      out[1] = a[1] * b;
      out[2] = a[2] * b;
      out[3] = a[3] * b;
      out[4] = a[4] * b;
      out[5] = a[5] * b;
      out[6] = a[6] * b;
      out[7] = a[7] * b;
      out[8] = a[8] * b;
      return out;
    }
    function multiplyScalarAndAdd(out, a, b, scale2) {
      out[0] = a[0] + b[0] * scale2;
      out[1] = a[1] + b[1] * scale2;
      out[2] = a[2] + b[2] * scale2;
      out[3] = a[3] + b[3] * scale2;
      out[4] = a[4] + b[4] * scale2;
      out[5] = a[5] + b[5] * scale2;
      out[6] = a[6] + b[6] * scale2;
      out[7] = a[7] + b[7] * scale2;
      out[8] = a[8] + b[8] * scale2;
      return out;
    }
    function exactEquals(a, b) {
      return a[0] === b[0] && a[1] === b[1] && a[2] === b[2] && a[3] === b[3] && a[4] === b[4] && a[5] === b[5] && a[6] === b[6] && a[7] === b[7] && a[8] === b[8];
    }
    function equals(a, b) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3], a4 = a[4], a5 = a[5], a6 = a[6], a7 = a[7], a8 = a[8];
      var b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3], b4 = b[4], b5 = b[5], b6 = b[6], b7 = b[7], b8 = b[8];
      return Math.abs(a0 - b0) <= glMatrix.EPSILON * Math.max(1, Math.abs(a0), Math.abs(b0)) && Math.abs(a1 - b1) <= glMatrix.EPSILON * Math.max(1, Math.abs(a1), Math.abs(b1)) && Math.abs(a2 - b2) <= glMatrix.EPSILON * Math.max(1, Math.abs(a2), Math.abs(b2)) && Math.abs(a3 - b3) <= glMatrix.EPSILON * Math.max(1, Math.abs(a3), Math.abs(b3)) && Math.abs(a4 - b4) <= glMatrix.EPSILON * Math.max(1, Math.abs(a4), Math.abs(b4)) && Math.abs(a5 - b5) <= glMatrix.EPSILON * Math.max(1, Math.abs(a5), Math.abs(b5)) && Math.abs(a6 - b6) <= glMatrix.EPSILON * Math.max(1, Math.abs(a6), Math.abs(b6)) && Math.abs(a7 - b7) <= glMatrix.EPSILON * Math.max(1, Math.abs(a7), Math.abs(b7)) && Math.abs(a8 - b8) <= glMatrix.EPSILON * Math.max(1, Math.abs(a8), Math.abs(b8));
    }
    var mul = exports2.mul = multiply;
    var sub = exports2.sub = subtract;
  }
});

// {TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/mat4.js
var require_mat4 = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/mat4.js"(exports2) {
    "use strict";
    function _typeof(o) {
      "@babel/helpers - typeof";
      return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function(o2) {
        return typeof o2;
      } : function(o2) {
        return o2 && "function" == typeof Symbol && o2.constructor === Symbol && o2 !== Symbol.prototype ? "symbol" : typeof o2;
      }, _typeof(o);
    }
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.add = add;
    exports2.adjoint = adjoint;
    exports2.clone = clone;
    exports2.copy = copy;
    exports2.create = create;
    exports2.decompose = decompose;
    exports2.determinant = determinant;
    exports2.equals = equals;
    exports2.exactEquals = exactEquals;
    exports2.frob = frob;
    exports2.fromQuat = fromQuat;
    exports2.fromQuat2 = fromQuat2;
    exports2.fromRotation = fromRotation;
    exports2.fromRotationTranslation = fromRotationTranslation;
    exports2.fromRotationTranslationScale = fromRotationTranslationScale;
    exports2.fromRotationTranslationScaleOrigin = fromRotationTranslationScaleOrigin;
    exports2.fromScaling = fromScaling;
    exports2.fromTranslation = fromTranslation;
    exports2.fromValues = fromValues;
    exports2.fromXRotation = fromXRotation;
    exports2.fromYRotation = fromYRotation;
    exports2.fromZRotation = fromZRotation;
    exports2.frustum = frustum;
    exports2.getRotation = getRotation;
    exports2.getScaling = getScaling;
    exports2.getTranslation = getTranslation;
    exports2.identity = identity;
    exports2.invert = invert;
    exports2.lookAt = lookAt;
    exports2.mul = void 0;
    exports2.multiply = multiply;
    exports2.multiplyScalar = multiplyScalar;
    exports2.multiplyScalarAndAdd = multiplyScalarAndAdd;
    exports2.ortho = void 0;
    exports2.orthoNO = orthoNO;
    exports2.orthoZO = orthoZO;
    exports2.perspective = void 0;
    exports2.perspectiveFromFieldOfView = perspectiveFromFieldOfView;
    exports2.perspectiveNO = perspectiveNO;
    exports2.perspectiveZO = perspectiveZO;
    exports2.rotate = rotate;
    exports2.rotateX = rotateX;
    exports2.rotateY = rotateY;
    exports2.rotateZ = rotateZ;
    exports2.scale = scale;
    exports2.set = set;
    exports2.str = str;
    exports2.sub = void 0;
    exports2.subtract = subtract;
    exports2.targetTo = targetTo;
    exports2.translate = translate;
    exports2.transpose = transpose;
    var glMatrix = _interopRequireWildcard(require_common());
    function _interopRequireWildcard(e, t) {
      if ("function" == typeof WeakMap) var r = /* @__PURE__ */ new WeakMap(), n = /* @__PURE__ */ new WeakMap();
      return (_interopRequireWildcard = function _interopRequireWildcard2(e2, t2) {
        if (!t2 && e2 && e2.__esModule) return e2;
        var o, i, f = { __proto__: null, "default": e2 };
        if (null === e2 || "object" != _typeof(e2) && "function" != typeof e2) return f;
        if (o = t2 ? n : r) {
          if (o.has(e2)) return o.get(e2);
          o.set(e2, f);
        }
        for (var _t in e2) "default" !== _t && {}.hasOwnProperty.call(e2, _t) && ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e2, _t)) && (i.get || i.set) ? o(f, _t, i) : f[_t] = e2[_t]);
        return f;
      })(e, t);
    }
    function create() {
      var out = new glMatrix.ARRAY_TYPE(16);
      if (glMatrix.ARRAY_TYPE != Float32Array) {
        out[1] = 0;
        out[2] = 0;
        out[3] = 0;
        out[4] = 0;
        out[6] = 0;
        out[7] = 0;
        out[8] = 0;
        out[9] = 0;
        out[11] = 0;
        out[12] = 0;
        out[13] = 0;
        out[14] = 0;
      }
      out[0] = 1;
      out[5] = 1;
      out[10] = 1;
      out[15] = 1;
      return out;
    }
    function clone(a) {
      var out = new glMatrix.ARRAY_TYPE(16);
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      out[3] = a[3];
      out[4] = a[4];
      out[5] = a[5];
      out[6] = a[6];
      out[7] = a[7];
      out[8] = a[8];
      out[9] = a[9];
      out[10] = a[10];
      out[11] = a[11];
      out[12] = a[12];
      out[13] = a[13];
      out[14] = a[14];
      out[15] = a[15];
      return out;
    }
    function copy(out, a) {
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      out[3] = a[3];
      out[4] = a[4];
      out[5] = a[5];
      out[6] = a[6];
      out[7] = a[7];
      out[8] = a[8];
      out[9] = a[9];
      out[10] = a[10];
      out[11] = a[11];
      out[12] = a[12];
      out[13] = a[13];
      out[14] = a[14];
      out[15] = a[15];
      return out;
    }
    function fromValues(m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33) {
      var out = new glMatrix.ARRAY_TYPE(16);
      out[0] = m00;
      out[1] = m01;
      out[2] = m02;
      out[3] = m03;
      out[4] = m10;
      out[5] = m11;
      out[6] = m12;
      out[7] = m13;
      out[8] = m20;
      out[9] = m21;
      out[10] = m22;
      out[11] = m23;
      out[12] = m30;
      out[13] = m31;
      out[14] = m32;
      out[15] = m33;
      return out;
    }
    function set(out, m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33) {
      out[0] = m00;
      out[1] = m01;
      out[2] = m02;
      out[3] = m03;
      out[4] = m10;
      out[5] = m11;
      out[6] = m12;
      out[7] = m13;
      out[8] = m20;
      out[9] = m21;
      out[10] = m22;
      out[11] = m23;
      out[12] = m30;
      out[13] = m31;
      out[14] = m32;
      out[15] = m33;
      return out;
    }
    function identity(out) {
      out[0] = 1;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = 1;
      out[6] = 0;
      out[7] = 0;
      out[8] = 0;
      out[9] = 0;
      out[10] = 1;
      out[11] = 0;
      out[12] = 0;
      out[13] = 0;
      out[14] = 0;
      out[15] = 1;
      return out;
    }
    function transpose(out, a) {
      if (out === a) {
        var a01 = a[1], a02 = a[2], a03 = a[3];
        var a12 = a[6], a13 = a[7];
        var a23 = a[11];
        out[1] = a[4];
        out[2] = a[8];
        out[3] = a[12];
        out[4] = a01;
        out[6] = a[9];
        out[7] = a[13];
        out[8] = a02;
        out[9] = a12;
        out[11] = a[14];
        out[12] = a03;
        out[13] = a13;
        out[14] = a23;
      } else {
        out[0] = a[0];
        out[1] = a[4];
        out[2] = a[8];
        out[3] = a[12];
        out[4] = a[1];
        out[5] = a[5];
        out[6] = a[9];
        out[7] = a[13];
        out[8] = a[2];
        out[9] = a[6];
        out[10] = a[10];
        out[11] = a[14];
        out[12] = a[3];
        out[13] = a[7];
        out[14] = a[11];
        out[15] = a[15];
      }
      return out;
    }
    function invert(out, a) {
      var a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
      var a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
      var a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
      var a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];
      var b00 = a00 * a11 - a01 * a10;
      var b01 = a00 * a12 - a02 * a10;
      var b02 = a00 * a13 - a03 * a10;
      var b03 = a01 * a12 - a02 * a11;
      var b04 = a01 * a13 - a03 * a11;
      var b05 = a02 * a13 - a03 * a12;
      var b06 = a20 * a31 - a21 * a30;
      var b07 = a20 * a32 - a22 * a30;
      var b08 = a20 * a33 - a23 * a30;
      var b09 = a21 * a32 - a22 * a31;
      var b10 = a21 * a33 - a23 * a31;
      var b11 = a22 * a33 - a23 * a32;
      var det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
      if (!det) {
        return null;
      }
      det = 1 / det;
      out[0] = (a11 * b11 - a12 * b10 + a13 * b09) * det;
      out[1] = (a02 * b10 - a01 * b11 - a03 * b09) * det;
      out[2] = (a31 * b05 - a32 * b04 + a33 * b03) * det;
      out[3] = (a22 * b04 - a21 * b05 - a23 * b03) * det;
      out[4] = (a12 * b08 - a10 * b11 - a13 * b07) * det;
      out[5] = (a00 * b11 - a02 * b08 + a03 * b07) * det;
      out[6] = (a32 * b02 - a30 * b05 - a33 * b01) * det;
      out[7] = (a20 * b05 - a22 * b02 + a23 * b01) * det;
      out[8] = (a10 * b10 - a11 * b08 + a13 * b06) * det;
      out[9] = (a01 * b08 - a00 * b10 - a03 * b06) * det;
      out[10] = (a30 * b04 - a31 * b02 + a33 * b00) * det;
      out[11] = (a21 * b02 - a20 * b04 - a23 * b00) * det;
      out[12] = (a11 * b07 - a10 * b09 - a12 * b06) * det;
      out[13] = (a00 * b09 - a01 * b07 + a02 * b06) * det;
      out[14] = (a31 * b01 - a30 * b03 - a32 * b00) * det;
      out[15] = (a20 * b03 - a21 * b01 + a22 * b00) * det;
      return out;
    }
    function adjoint(out, a) {
      var a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
      var a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
      var a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
      var a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];
      var b00 = a00 * a11 - a01 * a10;
      var b01 = a00 * a12 - a02 * a10;
      var b02 = a00 * a13 - a03 * a10;
      var b03 = a01 * a12 - a02 * a11;
      var b04 = a01 * a13 - a03 * a11;
      var b05 = a02 * a13 - a03 * a12;
      var b06 = a20 * a31 - a21 * a30;
      var b07 = a20 * a32 - a22 * a30;
      var b08 = a20 * a33 - a23 * a30;
      var b09 = a21 * a32 - a22 * a31;
      var b10 = a21 * a33 - a23 * a31;
      var b11 = a22 * a33 - a23 * a32;
      out[0] = a11 * b11 - a12 * b10 + a13 * b09;
      out[1] = a02 * b10 - a01 * b11 - a03 * b09;
      out[2] = a31 * b05 - a32 * b04 + a33 * b03;
      out[3] = a22 * b04 - a21 * b05 - a23 * b03;
      out[4] = a12 * b08 - a10 * b11 - a13 * b07;
      out[5] = a00 * b11 - a02 * b08 + a03 * b07;
      out[6] = a32 * b02 - a30 * b05 - a33 * b01;
      out[7] = a20 * b05 - a22 * b02 + a23 * b01;
      out[8] = a10 * b10 - a11 * b08 + a13 * b06;
      out[9] = a01 * b08 - a00 * b10 - a03 * b06;
      out[10] = a30 * b04 - a31 * b02 + a33 * b00;
      out[11] = a21 * b02 - a20 * b04 - a23 * b00;
      out[12] = a11 * b07 - a10 * b09 - a12 * b06;
      out[13] = a00 * b09 - a01 * b07 + a02 * b06;
      out[14] = a31 * b01 - a30 * b03 - a32 * b00;
      out[15] = a20 * b03 - a21 * b01 + a22 * b00;
      return out;
    }
    function determinant(a) {
      var a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
      var a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
      var a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
      var a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];
      var b0 = a00 * a11 - a01 * a10;
      var b1 = a00 * a12 - a02 * a10;
      var b2 = a01 * a12 - a02 * a11;
      var b3 = a20 * a31 - a21 * a30;
      var b4 = a20 * a32 - a22 * a30;
      var b5 = a21 * a32 - a22 * a31;
      var b6 = a00 * b5 - a01 * b4 + a02 * b3;
      var b7 = a10 * b5 - a11 * b4 + a12 * b3;
      var b8 = a20 * b2 - a21 * b1 + a22 * b0;
      var b9 = a30 * b2 - a31 * b1 + a32 * b0;
      return a13 * b6 - a03 * b7 + a33 * b8 - a23 * b9;
    }
    function multiply(out, a, b) {
      var a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
      var a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
      var a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
      var a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];
      var b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3];
      out[0] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
      out[1] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
      out[2] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
      out[3] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
      b0 = b[4];
      b1 = b[5];
      b2 = b[6];
      b3 = b[7];
      out[4] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
      out[5] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
      out[6] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
      out[7] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
      b0 = b[8];
      b1 = b[9];
      b2 = b[10];
      b3 = b[11];
      out[8] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
      out[9] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
      out[10] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
      out[11] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
      b0 = b[12];
      b1 = b[13];
      b2 = b[14];
      b3 = b[15];
      out[12] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
      out[13] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
      out[14] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
      out[15] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
      return out;
    }
    function translate(out, a, v) {
      var x = v[0], y = v[1], z = v[2];
      var a00, a01, a02, a03;
      var a10, a11, a12, a13;
      var a20, a21, a22, a23;
      if (a === out) {
        out[12] = a[0] * x + a[4] * y + a[8] * z + a[12];
        out[13] = a[1] * x + a[5] * y + a[9] * z + a[13];
        out[14] = a[2] * x + a[6] * y + a[10] * z + a[14];
        out[15] = a[3] * x + a[7] * y + a[11] * z + a[15];
      } else {
        a00 = a[0];
        a01 = a[1];
        a02 = a[2];
        a03 = a[3];
        a10 = a[4];
        a11 = a[5];
        a12 = a[6];
        a13 = a[7];
        a20 = a[8];
        a21 = a[9];
        a22 = a[10];
        a23 = a[11];
        out[0] = a00;
        out[1] = a01;
        out[2] = a02;
        out[3] = a03;
        out[4] = a10;
        out[5] = a11;
        out[6] = a12;
        out[7] = a13;
        out[8] = a20;
        out[9] = a21;
        out[10] = a22;
        out[11] = a23;
        out[12] = a00 * x + a10 * y + a20 * z + a[12];
        out[13] = a01 * x + a11 * y + a21 * z + a[13];
        out[14] = a02 * x + a12 * y + a22 * z + a[14];
        out[15] = a03 * x + a13 * y + a23 * z + a[15];
      }
      return out;
    }
    function scale(out, a, v) {
      var x = v[0], y = v[1], z = v[2];
      out[0] = a[0] * x;
      out[1] = a[1] * x;
      out[2] = a[2] * x;
      out[3] = a[3] * x;
      out[4] = a[4] * y;
      out[5] = a[5] * y;
      out[6] = a[6] * y;
      out[7] = a[7] * y;
      out[8] = a[8] * z;
      out[9] = a[9] * z;
      out[10] = a[10] * z;
      out[11] = a[11] * z;
      out[12] = a[12];
      out[13] = a[13];
      out[14] = a[14];
      out[15] = a[15];
      return out;
    }
    function rotate(out, a, rad, axis) {
      var x = axis[0], y = axis[1], z = axis[2];
      var len = Math.sqrt(x * x + y * y + z * z);
      var s, c, t;
      var a00, a01, a02, a03;
      var a10, a11, a12, a13;
      var a20, a21, a22, a23;
      var b00, b01, b02;
      var b10, b11, b12;
      var b20, b21, b22;
      if (len < glMatrix.EPSILON) {
        return null;
      }
      len = 1 / len;
      x *= len;
      y *= len;
      z *= len;
      s = Math.sin(rad);
      c = Math.cos(rad);
      t = 1 - c;
      a00 = a[0];
      a01 = a[1];
      a02 = a[2];
      a03 = a[3];
      a10 = a[4];
      a11 = a[5];
      a12 = a[6];
      a13 = a[7];
      a20 = a[8];
      a21 = a[9];
      a22 = a[10];
      a23 = a[11];
      b00 = x * x * t + c;
      b01 = y * x * t + z * s;
      b02 = z * x * t - y * s;
      b10 = x * y * t - z * s;
      b11 = y * y * t + c;
      b12 = z * y * t + x * s;
      b20 = x * z * t + y * s;
      b21 = y * z * t - x * s;
      b22 = z * z * t + c;
      out[0] = a00 * b00 + a10 * b01 + a20 * b02;
      out[1] = a01 * b00 + a11 * b01 + a21 * b02;
      out[2] = a02 * b00 + a12 * b01 + a22 * b02;
      out[3] = a03 * b00 + a13 * b01 + a23 * b02;
      out[4] = a00 * b10 + a10 * b11 + a20 * b12;
      out[5] = a01 * b10 + a11 * b11 + a21 * b12;
      out[6] = a02 * b10 + a12 * b11 + a22 * b12;
      out[7] = a03 * b10 + a13 * b11 + a23 * b12;
      out[8] = a00 * b20 + a10 * b21 + a20 * b22;
      out[9] = a01 * b20 + a11 * b21 + a21 * b22;
      out[10] = a02 * b20 + a12 * b21 + a22 * b22;
      out[11] = a03 * b20 + a13 * b21 + a23 * b22;
      if (a !== out) {
        out[12] = a[12];
        out[13] = a[13];
        out[14] = a[14];
        out[15] = a[15];
      }
      return out;
    }
    function rotateX(out, a, rad) {
      var s = Math.sin(rad);
      var c = Math.cos(rad);
      var a10 = a[4];
      var a11 = a[5];
      var a12 = a[6];
      var a13 = a[7];
      var a20 = a[8];
      var a21 = a[9];
      var a22 = a[10];
      var a23 = a[11];
      if (a !== out) {
        out[0] = a[0];
        out[1] = a[1];
        out[2] = a[2];
        out[3] = a[3];
        out[12] = a[12];
        out[13] = a[13];
        out[14] = a[14];
        out[15] = a[15];
      }
      out[4] = a10 * c + a20 * s;
      out[5] = a11 * c + a21 * s;
      out[6] = a12 * c + a22 * s;
      out[7] = a13 * c + a23 * s;
      out[8] = a20 * c - a10 * s;
      out[9] = a21 * c - a11 * s;
      out[10] = a22 * c - a12 * s;
      out[11] = a23 * c - a13 * s;
      return out;
    }
    function rotateY(out, a, rad) {
      var s = Math.sin(rad);
      var c = Math.cos(rad);
      var a00 = a[0];
      var a01 = a[1];
      var a02 = a[2];
      var a03 = a[3];
      var a20 = a[8];
      var a21 = a[9];
      var a22 = a[10];
      var a23 = a[11];
      if (a !== out) {
        out[4] = a[4];
        out[5] = a[5];
        out[6] = a[6];
        out[7] = a[7];
        out[12] = a[12];
        out[13] = a[13];
        out[14] = a[14];
        out[15] = a[15];
      }
      out[0] = a00 * c - a20 * s;
      out[1] = a01 * c - a21 * s;
      out[2] = a02 * c - a22 * s;
      out[3] = a03 * c - a23 * s;
      out[8] = a00 * s + a20 * c;
      out[9] = a01 * s + a21 * c;
      out[10] = a02 * s + a22 * c;
      out[11] = a03 * s + a23 * c;
      return out;
    }
    function rotateZ(out, a, rad) {
      var s = Math.sin(rad);
      var c = Math.cos(rad);
      var a00 = a[0];
      var a01 = a[1];
      var a02 = a[2];
      var a03 = a[3];
      var a10 = a[4];
      var a11 = a[5];
      var a12 = a[6];
      var a13 = a[7];
      if (a !== out) {
        out[8] = a[8];
        out[9] = a[9];
        out[10] = a[10];
        out[11] = a[11];
        out[12] = a[12];
        out[13] = a[13];
        out[14] = a[14];
        out[15] = a[15];
      }
      out[0] = a00 * c + a10 * s;
      out[1] = a01 * c + a11 * s;
      out[2] = a02 * c + a12 * s;
      out[3] = a03 * c + a13 * s;
      out[4] = a10 * c - a00 * s;
      out[5] = a11 * c - a01 * s;
      out[6] = a12 * c - a02 * s;
      out[7] = a13 * c - a03 * s;
      return out;
    }
    function fromTranslation(out, v) {
      out[0] = 1;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = 1;
      out[6] = 0;
      out[7] = 0;
      out[8] = 0;
      out[9] = 0;
      out[10] = 1;
      out[11] = 0;
      out[12] = v[0];
      out[13] = v[1];
      out[14] = v[2];
      out[15] = 1;
      return out;
    }
    function fromScaling(out, v) {
      out[0] = v[0];
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = v[1];
      out[6] = 0;
      out[7] = 0;
      out[8] = 0;
      out[9] = 0;
      out[10] = v[2];
      out[11] = 0;
      out[12] = 0;
      out[13] = 0;
      out[14] = 0;
      out[15] = 1;
      return out;
    }
    function fromRotation(out, rad, axis) {
      var x = axis[0], y = axis[1], z = axis[2];
      var len = Math.sqrt(x * x + y * y + z * z);
      var s, c, t;
      if (len < glMatrix.EPSILON) {
        return null;
      }
      len = 1 / len;
      x *= len;
      y *= len;
      z *= len;
      s = Math.sin(rad);
      c = Math.cos(rad);
      t = 1 - c;
      out[0] = x * x * t + c;
      out[1] = y * x * t + z * s;
      out[2] = z * x * t - y * s;
      out[3] = 0;
      out[4] = x * y * t - z * s;
      out[5] = y * y * t + c;
      out[6] = z * y * t + x * s;
      out[7] = 0;
      out[8] = x * z * t + y * s;
      out[9] = y * z * t - x * s;
      out[10] = z * z * t + c;
      out[11] = 0;
      out[12] = 0;
      out[13] = 0;
      out[14] = 0;
      out[15] = 1;
      return out;
    }
    function fromXRotation(out, rad) {
      var s = Math.sin(rad);
      var c = Math.cos(rad);
      out[0] = 1;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = c;
      out[6] = s;
      out[7] = 0;
      out[8] = 0;
      out[9] = -s;
      out[10] = c;
      out[11] = 0;
      out[12] = 0;
      out[13] = 0;
      out[14] = 0;
      out[15] = 1;
      return out;
    }
    function fromYRotation(out, rad) {
      var s = Math.sin(rad);
      var c = Math.cos(rad);
      out[0] = c;
      out[1] = 0;
      out[2] = -s;
      out[3] = 0;
      out[4] = 0;
      out[5] = 1;
      out[6] = 0;
      out[7] = 0;
      out[8] = s;
      out[9] = 0;
      out[10] = c;
      out[11] = 0;
      out[12] = 0;
      out[13] = 0;
      out[14] = 0;
      out[15] = 1;
      return out;
    }
    function fromZRotation(out, rad) {
      var s = Math.sin(rad);
      var c = Math.cos(rad);
      out[0] = c;
      out[1] = s;
      out[2] = 0;
      out[3] = 0;
      out[4] = -s;
      out[5] = c;
      out[6] = 0;
      out[7] = 0;
      out[8] = 0;
      out[9] = 0;
      out[10] = 1;
      out[11] = 0;
      out[12] = 0;
      out[13] = 0;
      out[14] = 0;
      out[15] = 1;
      return out;
    }
    function fromRotationTranslation(out, q, v) {
      var x = q[0], y = q[1], z = q[2], w = q[3];
      var x2 = x + x;
      var y2 = y + y;
      var z2 = z + z;
      var xx = x * x2;
      var xy = x * y2;
      var xz = x * z2;
      var yy = y * y2;
      var yz = y * z2;
      var zz = z * z2;
      var wx = w * x2;
      var wy = w * y2;
      var wz = w * z2;
      out[0] = 1 - (yy + zz);
      out[1] = xy + wz;
      out[2] = xz - wy;
      out[3] = 0;
      out[4] = xy - wz;
      out[5] = 1 - (xx + zz);
      out[6] = yz + wx;
      out[7] = 0;
      out[8] = xz + wy;
      out[9] = yz - wx;
      out[10] = 1 - (xx + yy);
      out[11] = 0;
      out[12] = v[0];
      out[13] = v[1];
      out[14] = v[2];
      out[15] = 1;
      return out;
    }
    function fromQuat2(out, a) {
      var translation = new glMatrix.ARRAY_TYPE(3);
      var bx = -a[0], by = -a[1], bz = -a[2], bw = a[3], ax = a[4], ay = a[5], az = a[6], aw = a[7];
      var magnitude = bx * bx + by * by + bz * bz + bw * bw;
      if (magnitude > 0) {
        translation[0] = (ax * bw + aw * bx + ay * bz - az * by) * 2 / magnitude;
        translation[1] = (ay * bw + aw * by + az * bx - ax * bz) * 2 / magnitude;
        translation[2] = (az * bw + aw * bz + ax * by - ay * bx) * 2 / magnitude;
      } else {
        translation[0] = (ax * bw + aw * bx + ay * bz - az * by) * 2;
        translation[1] = (ay * bw + aw * by + az * bx - ax * bz) * 2;
        translation[2] = (az * bw + aw * bz + ax * by - ay * bx) * 2;
      }
      fromRotationTranslation(out, a, translation);
      return out;
    }
    function getTranslation(out, mat) {
      out[0] = mat[12];
      out[1] = mat[13];
      out[2] = mat[14];
      return out;
    }
    function getScaling(out, mat) {
      var m11 = mat[0];
      var m12 = mat[1];
      var m13 = mat[2];
      var m21 = mat[4];
      var m22 = mat[5];
      var m23 = mat[6];
      var m31 = mat[8];
      var m32 = mat[9];
      var m33 = mat[10];
      out[0] = Math.sqrt(m11 * m11 + m12 * m12 + m13 * m13);
      out[1] = Math.sqrt(m21 * m21 + m22 * m22 + m23 * m23);
      out[2] = Math.sqrt(m31 * m31 + m32 * m32 + m33 * m33);
      return out;
    }
    function getRotation(out, mat) {
      var scaling = new glMatrix.ARRAY_TYPE(3);
      getScaling(scaling, mat);
      var is1 = 1 / scaling[0];
      var is2 = 1 / scaling[1];
      var is3 = 1 / scaling[2];
      var sm11 = mat[0] * is1;
      var sm12 = mat[1] * is2;
      var sm13 = mat[2] * is3;
      var sm21 = mat[4] * is1;
      var sm22 = mat[5] * is2;
      var sm23 = mat[6] * is3;
      var sm31 = mat[8] * is1;
      var sm32 = mat[9] * is2;
      var sm33 = mat[10] * is3;
      var trace = sm11 + sm22 + sm33;
      var S = 0;
      if (trace > 0) {
        S = Math.sqrt(trace + 1) * 2;
        out[3] = 0.25 * S;
        out[0] = (sm23 - sm32) / S;
        out[1] = (sm31 - sm13) / S;
        out[2] = (sm12 - sm21) / S;
      } else if (sm11 > sm22 && sm11 > sm33) {
        S = Math.sqrt(1 + sm11 - sm22 - sm33) * 2;
        out[3] = (sm23 - sm32) / S;
        out[0] = 0.25 * S;
        out[1] = (sm12 + sm21) / S;
        out[2] = (sm31 + sm13) / S;
      } else if (sm22 > sm33) {
        S = Math.sqrt(1 + sm22 - sm11 - sm33) * 2;
        out[3] = (sm31 - sm13) / S;
        out[0] = (sm12 + sm21) / S;
        out[1] = 0.25 * S;
        out[2] = (sm23 + sm32) / S;
      } else {
        S = Math.sqrt(1 + sm33 - sm11 - sm22) * 2;
        out[3] = (sm12 - sm21) / S;
        out[0] = (sm31 + sm13) / S;
        out[1] = (sm23 + sm32) / S;
        out[2] = 0.25 * S;
      }
      return out;
    }
    function decompose(out_r, out_t, out_s, mat) {
      out_t[0] = mat[12];
      out_t[1] = mat[13];
      out_t[2] = mat[14];
      var m11 = mat[0];
      var m12 = mat[1];
      var m13 = mat[2];
      var m21 = mat[4];
      var m22 = mat[5];
      var m23 = mat[6];
      var m31 = mat[8];
      var m32 = mat[9];
      var m33 = mat[10];
      out_s[0] = Math.sqrt(m11 * m11 + m12 * m12 + m13 * m13);
      out_s[1] = Math.sqrt(m21 * m21 + m22 * m22 + m23 * m23);
      out_s[2] = Math.sqrt(m31 * m31 + m32 * m32 + m33 * m33);
      var is1 = 1 / out_s[0];
      var is2 = 1 / out_s[1];
      var is3 = 1 / out_s[2];
      var sm11 = m11 * is1;
      var sm12 = m12 * is2;
      var sm13 = m13 * is3;
      var sm21 = m21 * is1;
      var sm22 = m22 * is2;
      var sm23 = m23 * is3;
      var sm31 = m31 * is1;
      var sm32 = m32 * is2;
      var sm33 = m33 * is3;
      var trace = sm11 + sm22 + sm33;
      var S = 0;
      if (trace > 0) {
        S = Math.sqrt(trace + 1) * 2;
        out_r[3] = 0.25 * S;
        out_r[0] = (sm23 - sm32) / S;
        out_r[1] = (sm31 - sm13) / S;
        out_r[2] = (sm12 - sm21) / S;
      } else if (sm11 > sm22 && sm11 > sm33) {
        S = Math.sqrt(1 + sm11 - sm22 - sm33) * 2;
        out_r[3] = (sm23 - sm32) / S;
        out_r[0] = 0.25 * S;
        out_r[1] = (sm12 + sm21) / S;
        out_r[2] = (sm31 + sm13) / S;
      } else if (sm22 > sm33) {
        S = Math.sqrt(1 + sm22 - sm11 - sm33) * 2;
        out_r[3] = (sm31 - sm13) / S;
        out_r[0] = (sm12 + sm21) / S;
        out_r[1] = 0.25 * S;
        out_r[2] = (sm23 + sm32) / S;
      } else {
        S = Math.sqrt(1 + sm33 - sm11 - sm22) * 2;
        out_r[3] = (sm12 - sm21) / S;
        out_r[0] = (sm31 + sm13) / S;
        out_r[1] = (sm23 + sm32) / S;
        out_r[2] = 0.25 * S;
      }
      return out_r;
    }
    function fromRotationTranslationScale(out, q, v, s) {
      var x = q[0], y = q[1], z = q[2], w = q[3];
      var x2 = x + x;
      var y2 = y + y;
      var z2 = z + z;
      var xx = x * x2;
      var xy = x * y2;
      var xz = x * z2;
      var yy = y * y2;
      var yz = y * z2;
      var zz = z * z2;
      var wx = w * x2;
      var wy = w * y2;
      var wz = w * z2;
      var sx = s[0];
      var sy = s[1];
      var sz = s[2];
      out[0] = (1 - (yy + zz)) * sx;
      out[1] = (xy + wz) * sx;
      out[2] = (xz - wy) * sx;
      out[3] = 0;
      out[4] = (xy - wz) * sy;
      out[5] = (1 - (xx + zz)) * sy;
      out[6] = (yz + wx) * sy;
      out[7] = 0;
      out[8] = (xz + wy) * sz;
      out[9] = (yz - wx) * sz;
      out[10] = (1 - (xx + yy)) * sz;
      out[11] = 0;
      out[12] = v[0];
      out[13] = v[1];
      out[14] = v[2];
      out[15] = 1;
      return out;
    }
    function fromRotationTranslationScaleOrigin(out, q, v, s, o) {
      var x = q[0], y = q[1], z = q[2], w = q[3];
      var x2 = x + x;
      var y2 = y + y;
      var z2 = z + z;
      var xx = x * x2;
      var xy = x * y2;
      var xz = x * z2;
      var yy = y * y2;
      var yz = y * z2;
      var zz = z * z2;
      var wx = w * x2;
      var wy = w * y2;
      var wz = w * z2;
      var sx = s[0];
      var sy = s[1];
      var sz = s[2];
      var ox = o[0];
      var oy = o[1];
      var oz = o[2];
      var out0 = (1 - (yy + zz)) * sx;
      var out1 = (xy + wz) * sx;
      var out2 = (xz - wy) * sx;
      var out4 = (xy - wz) * sy;
      var out5 = (1 - (xx + zz)) * sy;
      var out6 = (yz + wx) * sy;
      var out8 = (xz + wy) * sz;
      var out9 = (yz - wx) * sz;
      var out10 = (1 - (xx + yy)) * sz;
      out[0] = out0;
      out[1] = out1;
      out[2] = out2;
      out[3] = 0;
      out[4] = out4;
      out[5] = out5;
      out[6] = out6;
      out[7] = 0;
      out[8] = out8;
      out[9] = out9;
      out[10] = out10;
      out[11] = 0;
      out[12] = v[0] + ox - (out0 * ox + out4 * oy + out8 * oz);
      out[13] = v[1] + oy - (out1 * ox + out5 * oy + out9 * oz);
      out[14] = v[2] + oz - (out2 * ox + out6 * oy + out10 * oz);
      out[15] = 1;
      return out;
    }
    function fromQuat(out, q) {
      var x = q[0], y = q[1], z = q[2], w = q[3];
      var x2 = x + x;
      var y2 = y + y;
      var z2 = z + z;
      var xx = x * x2;
      var yx = y * x2;
      var yy = y * y2;
      var zx = z * x2;
      var zy = z * y2;
      var zz = z * z2;
      var wx = w * x2;
      var wy = w * y2;
      var wz = w * z2;
      out[0] = 1 - yy - zz;
      out[1] = yx + wz;
      out[2] = zx - wy;
      out[3] = 0;
      out[4] = yx - wz;
      out[5] = 1 - xx - zz;
      out[6] = zy + wx;
      out[7] = 0;
      out[8] = zx + wy;
      out[9] = zy - wx;
      out[10] = 1 - xx - yy;
      out[11] = 0;
      out[12] = 0;
      out[13] = 0;
      out[14] = 0;
      out[15] = 1;
      return out;
    }
    function frustum(out, left, right, bottom, top, near, far) {
      var rl = 1 / (right - left);
      var tb = 1 / (top - bottom);
      var nf = 1 / (near - far);
      out[0] = near * 2 * rl;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = near * 2 * tb;
      out[6] = 0;
      out[7] = 0;
      out[8] = (right + left) * rl;
      out[9] = (top + bottom) * tb;
      out[10] = (far + near) * nf;
      out[11] = -1;
      out[12] = 0;
      out[13] = 0;
      out[14] = far * near * 2 * nf;
      out[15] = 0;
      return out;
    }
    function perspectiveNO(out, fovy, aspect, near, far) {
      var f = 1 / Math.tan(fovy / 2);
      out[0] = f / aspect;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = f;
      out[6] = 0;
      out[7] = 0;
      out[8] = 0;
      out[9] = 0;
      out[11] = -1;
      out[12] = 0;
      out[13] = 0;
      out[15] = 0;
      if (far != null && far !== Infinity) {
        var nf = 1 / (near - far);
        out[10] = (far + near) * nf;
        out[14] = 2 * far * near * nf;
      } else {
        out[10] = -1;
        out[14] = -2 * near;
      }
      return out;
    }
    var perspective = exports2.perspective = perspectiveNO;
    function perspectiveZO(out, fovy, aspect, near, far) {
      var f = 1 / Math.tan(fovy / 2);
      out[0] = f / aspect;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = f;
      out[6] = 0;
      out[7] = 0;
      out[8] = 0;
      out[9] = 0;
      out[11] = -1;
      out[12] = 0;
      out[13] = 0;
      out[15] = 0;
      if (far != null && far !== Infinity) {
        var nf = 1 / (near - far);
        out[10] = far * nf;
        out[14] = far * near * nf;
      } else {
        out[10] = -1;
        out[14] = -near;
      }
      return out;
    }
    function perspectiveFromFieldOfView(out, fov, near, far) {
      var upTan = Math.tan(fov.upDegrees * Math.PI / 180);
      var downTan = Math.tan(fov.downDegrees * Math.PI / 180);
      var leftTan = Math.tan(fov.leftDegrees * Math.PI / 180);
      var rightTan = Math.tan(fov.rightDegrees * Math.PI / 180);
      var xScale = 2 / (leftTan + rightTan);
      var yScale = 2 / (upTan + downTan);
      out[0] = xScale;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = yScale;
      out[6] = 0;
      out[7] = 0;
      out[8] = -((leftTan - rightTan) * xScale * 0.5);
      out[9] = (upTan - downTan) * yScale * 0.5;
      out[10] = far / (near - far);
      out[11] = -1;
      out[12] = 0;
      out[13] = 0;
      out[14] = far * near / (near - far);
      out[15] = 0;
      return out;
    }
    function orthoNO(out, left, right, bottom, top, near, far) {
      var lr = 1 / (left - right);
      var bt = 1 / (bottom - top);
      var nf = 1 / (near - far);
      out[0] = -2 * lr;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = -2 * bt;
      out[6] = 0;
      out[7] = 0;
      out[8] = 0;
      out[9] = 0;
      out[10] = 2 * nf;
      out[11] = 0;
      out[12] = (left + right) * lr;
      out[13] = (top + bottom) * bt;
      out[14] = (far + near) * nf;
      out[15] = 1;
      return out;
    }
    var ortho = exports2.ortho = orthoNO;
    function orthoZO(out, left, right, bottom, top, near, far) {
      var lr = 1 / (left - right);
      var bt = 1 / (bottom - top);
      var nf = 1 / (near - far);
      out[0] = -2 * lr;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = -2 * bt;
      out[6] = 0;
      out[7] = 0;
      out[8] = 0;
      out[9] = 0;
      out[10] = nf;
      out[11] = 0;
      out[12] = (left + right) * lr;
      out[13] = (top + bottom) * bt;
      out[14] = near * nf;
      out[15] = 1;
      return out;
    }
    function lookAt(out, eye, center, up) {
      var x0, x1, x2, y0, y1, y2, z0, z1, z2, len;
      var eyex = eye[0];
      var eyey = eye[1];
      var eyez = eye[2];
      var upx = up[0];
      var upy = up[1];
      var upz = up[2];
      var centerx = center[0];
      var centery = center[1];
      var centerz = center[2];
      if (Math.abs(eyex - centerx) < glMatrix.EPSILON && Math.abs(eyey - centery) < glMatrix.EPSILON && Math.abs(eyez - centerz) < glMatrix.EPSILON) {
        return identity(out);
      }
      z0 = eyex - centerx;
      z1 = eyey - centery;
      z2 = eyez - centerz;
      len = 1 / Math.sqrt(z0 * z0 + z1 * z1 + z2 * z2);
      z0 *= len;
      z1 *= len;
      z2 *= len;
      x0 = upy * z2 - upz * z1;
      x1 = upz * z0 - upx * z2;
      x2 = upx * z1 - upy * z0;
      len = Math.sqrt(x0 * x0 + x1 * x1 + x2 * x2);
      if (!len) {
        x0 = 0;
        x1 = 0;
        x2 = 0;
      } else {
        len = 1 / len;
        x0 *= len;
        x1 *= len;
        x2 *= len;
      }
      y0 = z1 * x2 - z2 * x1;
      y1 = z2 * x0 - z0 * x2;
      y2 = z0 * x1 - z1 * x0;
      len = Math.sqrt(y0 * y0 + y1 * y1 + y2 * y2);
      if (!len) {
        y0 = 0;
        y1 = 0;
        y2 = 0;
      } else {
        len = 1 / len;
        y0 *= len;
        y1 *= len;
        y2 *= len;
      }
      out[0] = x0;
      out[1] = y0;
      out[2] = z0;
      out[3] = 0;
      out[4] = x1;
      out[5] = y1;
      out[6] = z1;
      out[7] = 0;
      out[8] = x2;
      out[9] = y2;
      out[10] = z2;
      out[11] = 0;
      out[12] = -(x0 * eyex + x1 * eyey + x2 * eyez);
      out[13] = -(y0 * eyex + y1 * eyey + y2 * eyez);
      out[14] = -(z0 * eyex + z1 * eyey + z2 * eyez);
      out[15] = 1;
      return out;
    }
    function targetTo(out, eye, target, up) {
      var eyex = eye[0], eyey = eye[1], eyez = eye[2], upx = up[0], upy = up[1], upz = up[2];
      var z0 = eyex - target[0], z1 = eyey - target[1], z2 = eyez - target[2];
      var len = z0 * z0 + z1 * z1 + z2 * z2;
      if (len > 0) {
        len = 1 / Math.sqrt(len);
        z0 *= len;
        z1 *= len;
        z2 *= len;
      }
      var x0 = upy * z2 - upz * z1, x1 = upz * z0 - upx * z2, x2 = upx * z1 - upy * z0;
      len = x0 * x0 + x1 * x1 + x2 * x2;
      if (len > 0) {
        len = 1 / Math.sqrt(len);
        x0 *= len;
        x1 *= len;
        x2 *= len;
      }
      out[0] = x0;
      out[1] = x1;
      out[2] = x2;
      out[3] = 0;
      out[4] = z1 * x2 - z2 * x1;
      out[5] = z2 * x0 - z0 * x2;
      out[6] = z0 * x1 - z1 * x0;
      out[7] = 0;
      out[8] = z0;
      out[9] = z1;
      out[10] = z2;
      out[11] = 0;
      out[12] = eyex;
      out[13] = eyey;
      out[14] = eyez;
      out[15] = 1;
      return out;
    }
    function str(a) {
      return "mat4(" + a[0] + ", " + a[1] + ", " + a[2] + ", " + a[3] + ", " + a[4] + ", " + a[5] + ", " + a[6] + ", " + a[7] + ", " + a[8] + ", " + a[9] + ", " + a[10] + ", " + a[11] + ", " + a[12] + ", " + a[13] + ", " + a[14] + ", " + a[15] + ")";
    }
    function frob(a) {
      return Math.sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2] + a[3] * a[3] + a[4] * a[4] + a[5] * a[5] + a[6] * a[6] + a[7] * a[7] + a[8] * a[8] + a[9] * a[9] + a[10] * a[10] + a[11] * a[11] + a[12] * a[12] + a[13] * a[13] + a[14] * a[14] + a[15] * a[15]);
    }
    function add(out, a, b) {
      out[0] = a[0] + b[0];
      out[1] = a[1] + b[1];
      out[2] = a[2] + b[2];
      out[3] = a[3] + b[3];
      out[4] = a[4] + b[4];
      out[5] = a[5] + b[5];
      out[6] = a[6] + b[6];
      out[7] = a[7] + b[7];
      out[8] = a[8] + b[8];
      out[9] = a[9] + b[9];
      out[10] = a[10] + b[10];
      out[11] = a[11] + b[11];
      out[12] = a[12] + b[12];
      out[13] = a[13] + b[13];
      out[14] = a[14] + b[14];
      out[15] = a[15] + b[15];
      return out;
    }
    function subtract(out, a, b) {
      out[0] = a[0] - b[0];
      out[1] = a[1] - b[1];
      out[2] = a[2] - b[2];
      out[3] = a[3] - b[3];
      out[4] = a[4] - b[4];
      out[5] = a[5] - b[5];
      out[6] = a[6] - b[6];
      out[7] = a[7] - b[7];
      out[8] = a[8] - b[8];
      out[9] = a[9] - b[9];
      out[10] = a[10] - b[10];
      out[11] = a[11] - b[11];
      out[12] = a[12] - b[12];
      out[13] = a[13] - b[13];
      out[14] = a[14] - b[14];
      out[15] = a[15] - b[15];
      return out;
    }
    function multiplyScalar(out, a, b) {
      out[0] = a[0] * b;
      out[1] = a[1] * b;
      out[2] = a[2] * b;
      out[3] = a[3] * b;
      out[4] = a[4] * b;
      out[5] = a[5] * b;
      out[6] = a[6] * b;
      out[7] = a[7] * b;
      out[8] = a[8] * b;
      out[9] = a[9] * b;
      out[10] = a[10] * b;
      out[11] = a[11] * b;
      out[12] = a[12] * b;
      out[13] = a[13] * b;
      out[14] = a[14] * b;
      out[15] = a[15] * b;
      return out;
    }
    function multiplyScalarAndAdd(out, a, b, scale2) {
      out[0] = a[0] + b[0] * scale2;
      out[1] = a[1] + b[1] * scale2;
      out[2] = a[2] + b[2] * scale2;
      out[3] = a[3] + b[3] * scale2;
      out[4] = a[4] + b[4] * scale2;
      out[5] = a[5] + b[5] * scale2;
      out[6] = a[6] + b[6] * scale2;
      out[7] = a[7] + b[7] * scale2;
      out[8] = a[8] + b[8] * scale2;
      out[9] = a[9] + b[9] * scale2;
      out[10] = a[10] + b[10] * scale2;
      out[11] = a[11] + b[11] * scale2;
      out[12] = a[12] + b[12] * scale2;
      out[13] = a[13] + b[13] * scale2;
      out[14] = a[14] + b[14] * scale2;
      out[15] = a[15] + b[15] * scale2;
      return out;
    }
    function exactEquals(a, b) {
      return a[0] === b[0] && a[1] === b[1] && a[2] === b[2] && a[3] === b[3] && a[4] === b[4] && a[5] === b[5] && a[6] === b[6] && a[7] === b[7] && a[8] === b[8] && a[9] === b[9] && a[10] === b[10] && a[11] === b[11] && a[12] === b[12] && a[13] === b[13] && a[14] === b[14] && a[15] === b[15];
    }
    function equals(a, b) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3];
      var a4 = a[4], a5 = a[5], a6 = a[6], a7 = a[7];
      var a8 = a[8], a9 = a[9], a10 = a[10], a11 = a[11];
      var a12 = a[12], a13 = a[13], a14 = a[14], a15 = a[15];
      var b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3];
      var b4 = b[4], b5 = b[5], b6 = b[6], b7 = b[7];
      var b8 = b[8], b9 = b[9], b10 = b[10], b11 = b[11];
      var b12 = b[12], b13 = b[13], b14 = b[14], b15 = b[15];
      return Math.abs(a0 - b0) <= glMatrix.EPSILON * Math.max(1, Math.abs(a0), Math.abs(b0)) && Math.abs(a1 - b1) <= glMatrix.EPSILON * Math.max(1, Math.abs(a1), Math.abs(b1)) && Math.abs(a2 - b2) <= glMatrix.EPSILON * Math.max(1, Math.abs(a2), Math.abs(b2)) && Math.abs(a3 - b3) <= glMatrix.EPSILON * Math.max(1, Math.abs(a3), Math.abs(b3)) && Math.abs(a4 - b4) <= glMatrix.EPSILON * Math.max(1, Math.abs(a4), Math.abs(b4)) && Math.abs(a5 - b5) <= glMatrix.EPSILON * Math.max(1, Math.abs(a5), Math.abs(b5)) && Math.abs(a6 - b6) <= glMatrix.EPSILON * Math.max(1, Math.abs(a6), Math.abs(b6)) && Math.abs(a7 - b7) <= glMatrix.EPSILON * Math.max(1, Math.abs(a7), Math.abs(b7)) && Math.abs(a8 - b8) <= glMatrix.EPSILON * Math.max(1, Math.abs(a8), Math.abs(b8)) && Math.abs(a9 - b9) <= glMatrix.EPSILON * Math.max(1, Math.abs(a9), Math.abs(b9)) && Math.abs(a10 - b10) <= glMatrix.EPSILON * Math.max(1, Math.abs(a10), Math.abs(b10)) && Math.abs(a11 - b11) <= glMatrix.EPSILON * Math.max(1, Math.abs(a11), Math.abs(b11)) && Math.abs(a12 - b12) <= glMatrix.EPSILON * Math.max(1, Math.abs(a12), Math.abs(b12)) && Math.abs(a13 - b13) <= glMatrix.EPSILON * Math.max(1, Math.abs(a13), Math.abs(b13)) && Math.abs(a14 - b14) <= glMatrix.EPSILON * Math.max(1, Math.abs(a14), Math.abs(b14)) && Math.abs(a15 - b15) <= glMatrix.EPSILON * Math.max(1, Math.abs(a15), Math.abs(b15));
    }
    var mul = exports2.mul = multiply;
    var sub = exports2.sub = subtract;
  }
});

// {TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/vec3.js
var require_vec3 = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/vec3.js"(exports2) {
    "use strict";
    function _typeof(o) {
      "@babel/helpers - typeof";
      return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function(o2) {
        return typeof o2;
      } : function(o2) {
        return o2 && "function" == typeof Symbol && o2.constructor === Symbol && o2 !== Symbol.prototype ? "symbol" : typeof o2;
      }, _typeof(o);
    }
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.add = add;
    exports2.angle = angle;
    exports2.bezier = bezier;
    exports2.ceil = ceil;
    exports2.clone = clone;
    exports2.copy = copy;
    exports2.create = create;
    exports2.cross = cross;
    exports2.dist = void 0;
    exports2.distance = distance;
    exports2.div = void 0;
    exports2.divide = divide;
    exports2.dot = dot;
    exports2.equals = equals;
    exports2.exactEquals = exactEquals;
    exports2.floor = floor;
    exports2.forEach = void 0;
    exports2.fromValues = fromValues;
    exports2.hermite = hermite;
    exports2.inverse = inverse;
    exports2.len = void 0;
    exports2.length = length;
    exports2.lerp = lerp;
    exports2.max = max;
    exports2.min = min;
    exports2.mul = void 0;
    exports2.multiply = multiply;
    exports2.negate = negate;
    exports2.normalize = normalize;
    exports2.random = random;
    exports2.rotateX = rotateX;
    exports2.rotateY = rotateY;
    exports2.rotateZ = rotateZ;
    exports2.round = round;
    exports2.scale = scale;
    exports2.scaleAndAdd = scaleAndAdd;
    exports2.set = set;
    exports2.slerp = slerp;
    exports2.sqrLen = exports2.sqrDist = void 0;
    exports2.squaredDistance = squaredDistance;
    exports2.squaredLength = squaredLength;
    exports2.str = str;
    exports2.sub = void 0;
    exports2.subtract = subtract;
    exports2.transformMat3 = transformMat3;
    exports2.transformMat4 = transformMat4;
    exports2.transformQuat = transformQuat;
    exports2.zero = zero;
    var glMatrix = _interopRequireWildcard(require_common());
    function _interopRequireWildcard(e, t) {
      if ("function" == typeof WeakMap) var r = /* @__PURE__ */ new WeakMap(), n = /* @__PURE__ */ new WeakMap();
      return (_interopRequireWildcard = function _interopRequireWildcard2(e2, t2) {
        if (!t2 && e2 && e2.__esModule) return e2;
        var o, i, f = { __proto__: null, "default": e2 };
        if (null === e2 || "object" != _typeof(e2) && "function" != typeof e2) return f;
        if (o = t2 ? n : r) {
          if (o.has(e2)) return o.get(e2);
          o.set(e2, f);
        }
        for (var _t in e2) "default" !== _t && {}.hasOwnProperty.call(e2, _t) && ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e2, _t)) && (i.get || i.set) ? o(f, _t, i) : f[_t] = e2[_t]);
        return f;
      })(e, t);
    }
    function create() {
      var out = new glMatrix.ARRAY_TYPE(3);
      if (glMatrix.ARRAY_TYPE != Float32Array) {
        out[0] = 0;
        out[1] = 0;
        out[2] = 0;
      }
      return out;
    }
    function clone(a) {
      var out = new glMatrix.ARRAY_TYPE(3);
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      return out;
    }
    function length(a) {
      var x = a[0];
      var y = a[1];
      var z = a[2];
      return Math.sqrt(x * x + y * y + z * z);
    }
    function fromValues(x, y, z) {
      var out = new glMatrix.ARRAY_TYPE(3);
      out[0] = x;
      out[1] = y;
      out[2] = z;
      return out;
    }
    function copy(out, a) {
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      return out;
    }
    function set(out, x, y, z) {
      out[0] = x;
      out[1] = y;
      out[2] = z;
      return out;
    }
    function add(out, a, b) {
      out[0] = a[0] + b[0];
      out[1] = a[1] + b[1];
      out[2] = a[2] + b[2];
      return out;
    }
    function subtract(out, a, b) {
      out[0] = a[0] - b[0];
      out[1] = a[1] - b[1];
      out[2] = a[2] - b[2];
      return out;
    }
    function multiply(out, a, b) {
      out[0] = a[0] * b[0];
      out[1] = a[1] * b[1];
      out[2] = a[2] * b[2];
      return out;
    }
    function divide(out, a, b) {
      out[0] = a[0] / b[0];
      out[1] = a[1] / b[1];
      out[2] = a[2] / b[2];
      return out;
    }
    function ceil(out, a) {
      out[0] = Math.ceil(a[0]);
      out[1] = Math.ceil(a[1]);
      out[2] = Math.ceil(a[2]);
      return out;
    }
    function floor(out, a) {
      out[0] = Math.floor(a[0]);
      out[1] = Math.floor(a[1]);
      out[2] = Math.floor(a[2]);
      return out;
    }
    function min(out, a, b) {
      out[0] = Math.min(a[0], b[0]);
      out[1] = Math.min(a[1], b[1]);
      out[2] = Math.min(a[2], b[2]);
      return out;
    }
    function max(out, a, b) {
      out[0] = Math.max(a[0], b[0]);
      out[1] = Math.max(a[1], b[1]);
      out[2] = Math.max(a[2], b[2]);
      return out;
    }
    function round(out, a) {
      out[0] = glMatrix.round(a[0]);
      out[1] = glMatrix.round(a[1]);
      out[2] = glMatrix.round(a[2]);
      return out;
    }
    function scale(out, a, b) {
      out[0] = a[0] * b;
      out[1] = a[1] * b;
      out[2] = a[2] * b;
      return out;
    }
    function scaleAndAdd(out, a, b, scale2) {
      out[0] = a[0] + b[0] * scale2;
      out[1] = a[1] + b[1] * scale2;
      out[2] = a[2] + b[2] * scale2;
      return out;
    }
    function distance(a, b) {
      var x = b[0] - a[0];
      var y = b[1] - a[1];
      var z = b[2] - a[2];
      return Math.sqrt(x * x + y * y + z * z);
    }
    function squaredDistance(a, b) {
      var x = b[0] - a[0];
      var y = b[1] - a[1];
      var z = b[2] - a[2];
      return x * x + y * y + z * z;
    }
    function squaredLength(a) {
      var x = a[0];
      var y = a[1];
      var z = a[2];
      return x * x + y * y + z * z;
    }
    function negate(out, a) {
      out[0] = -a[0];
      out[1] = -a[1];
      out[2] = -a[2];
      return out;
    }
    function inverse(out, a) {
      out[0] = 1 / a[0];
      out[1] = 1 / a[1];
      out[2] = 1 / a[2];
      return out;
    }
    function normalize(out, a) {
      var x = a[0];
      var y = a[1];
      var z = a[2];
      var len2 = x * x + y * y + z * z;
      if (len2 > 0) {
        len2 = 1 / Math.sqrt(len2);
      }
      out[0] = a[0] * len2;
      out[1] = a[1] * len2;
      out[2] = a[2] * len2;
      return out;
    }
    function dot(a, b) {
      return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    }
    function cross(out, a, b) {
      var ax = a[0], ay = a[1], az = a[2];
      var bx = b[0], by = b[1], bz = b[2];
      out[0] = ay * bz - az * by;
      out[1] = az * bx - ax * bz;
      out[2] = ax * by - ay * bx;
      return out;
    }
    function lerp(out, a, b, t) {
      var ax = a[0];
      var ay = a[1];
      var az = a[2];
      out[0] = ax + t * (b[0] - ax);
      out[1] = ay + t * (b[1] - ay);
      out[2] = az + t * (b[2] - az);
      return out;
    }
    function slerp(out, a, b, t) {
      var angle2 = Math.acos(Math.min(Math.max(dot(a, b), -1), 1));
      var sinTotal = Math.sin(angle2);
      var ratioA = Math.sin((1 - t) * angle2) / sinTotal;
      var ratioB = Math.sin(t * angle2) / sinTotal;
      out[0] = ratioA * a[0] + ratioB * b[0];
      out[1] = ratioA * a[1] + ratioB * b[1];
      out[2] = ratioA * a[2] + ratioB * b[2];
      return out;
    }
    function hermite(out, a, b, c, d2, t) {
      var factorTimes2 = t * t;
      var factor1 = factorTimes2 * (2 * t - 3) + 1;
      var factor2 = factorTimes2 * (t - 2) + t;
      var factor3 = factorTimes2 * (t - 1);
      var factor4 = factorTimes2 * (3 - 2 * t);
      out[0] = a[0] * factor1 + b[0] * factor2 + c[0] * factor3 + d2[0] * factor4;
      out[1] = a[1] * factor1 + b[1] * factor2 + c[1] * factor3 + d2[1] * factor4;
      out[2] = a[2] * factor1 + b[2] * factor2 + c[2] * factor3 + d2[2] * factor4;
      return out;
    }
    function bezier(out, a, b, c, d2, t) {
      var inverseFactor = 1 - t;
      var inverseFactorTimesTwo = inverseFactor * inverseFactor;
      var factorTimes2 = t * t;
      var factor1 = inverseFactorTimesTwo * inverseFactor;
      var factor2 = 3 * t * inverseFactorTimesTwo;
      var factor3 = 3 * factorTimes2 * inverseFactor;
      var factor4 = factorTimes2 * t;
      out[0] = a[0] * factor1 + b[0] * factor2 + c[0] * factor3 + d2[0] * factor4;
      out[1] = a[1] * factor1 + b[1] * factor2 + c[1] * factor3 + d2[1] * factor4;
      out[2] = a[2] * factor1 + b[2] * factor2 + c[2] * factor3 + d2[2] * factor4;
      return out;
    }
    function random(out, scale2) {
      scale2 = scale2 === void 0 ? 1 : scale2;
      var r = glMatrix.RANDOM() * 2 * Math.PI;
      var z = glMatrix.RANDOM() * 2 - 1;
      var zScale = Math.sqrt(1 - z * z) * scale2;
      out[0] = Math.cos(r) * zScale;
      out[1] = Math.sin(r) * zScale;
      out[2] = z * scale2;
      return out;
    }
    function transformMat4(out, a, m) {
      var x = a[0], y = a[1], z = a[2];
      var w = m[3] * x + m[7] * y + m[11] * z + m[15];
      w = w || 1;
      out[0] = (m[0] * x + m[4] * y + m[8] * z + m[12]) / w;
      out[1] = (m[1] * x + m[5] * y + m[9] * z + m[13]) / w;
      out[2] = (m[2] * x + m[6] * y + m[10] * z + m[14]) / w;
      return out;
    }
    function transformMat3(out, a, m) {
      var x = a[0], y = a[1], z = a[2];
      out[0] = x * m[0] + y * m[3] + z * m[6];
      out[1] = x * m[1] + y * m[4] + z * m[7];
      out[2] = x * m[2] + y * m[5] + z * m[8];
      return out;
    }
    function transformQuat(out, a, q) {
      var qx = q[0], qy = q[1], qz = q[2], qw = q[3];
      var vx = a[0], vy = a[1], vz = a[2];
      var tx = qy * vz - qz * vy;
      var ty = qz * vx - qx * vz;
      var tz = qx * vy - qy * vx;
      tx = tx + tx;
      ty = ty + ty;
      tz = tz + tz;
      out[0] = vx + qw * tx + qy * tz - qz * ty;
      out[1] = vy + qw * ty + qz * tx - qx * tz;
      out[2] = vz + qw * tz + qx * ty - qy * tx;
      return out;
    }
    function rotateX(out, a, b, rad) {
      var p = [], r = [];
      p[0] = a[0] - b[0];
      p[1] = a[1] - b[1];
      p[2] = a[2] - b[2];
      r[0] = p[0];
      r[1] = p[1] * Math.cos(rad) - p[2] * Math.sin(rad);
      r[2] = p[1] * Math.sin(rad) + p[2] * Math.cos(rad);
      out[0] = r[0] + b[0];
      out[1] = r[1] + b[1];
      out[2] = r[2] + b[2];
      return out;
    }
    function rotateY(out, a, b, rad) {
      var p = [], r = [];
      p[0] = a[0] - b[0];
      p[1] = a[1] - b[1];
      p[2] = a[2] - b[2];
      r[0] = p[2] * Math.sin(rad) + p[0] * Math.cos(rad);
      r[1] = p[1];
      r[2] = p[2] * Math.cos(rad) - p[0] * Math.sin(rad);
      out[0] = r[0] + b[0];
      out[1] = r[1] + b[1];
      out[2] = r[2] + b[2];
      return out;
    }
    function rotateZ(out, a, b, rad) {
      var p = [], r = [];
      p[0] = a[0] - b[0];
      p[1] = a[1] - b[1];
      p[2] = a[2] - b[2];
      r[0] = p[0] * Math.cos(rad) - p[1] * Math.sin(rad);
      r[1] = p[0] * Math.sin(rad) + p[1] * Math.cos(rad);
      r[2] = p[2];
      out[0] = r[0] + b[0];
      out[1] = r[1] + b[1];
      out[2] = r[2] + b[2];
      return out;
    }
    function angle(a, b) {
      var ax = a[0], ay = a[1], az = a[2], bx = b[0], by = b[1], bz = b[2], mag = Math.sqrt((ax * ax + ay * ay + az * az) * (bx * bx + by * by + bz * bz)), cosine = mag && dot(a, b) / mag;
      return Math.acos(Math.min(Math.max(cosine, -1), 1));
    }
    function zero(out) {
      out[0] = 0;
      out[1] = 0;
      out[2] = 0;
      return out;
    }
    function str(a) {
      return "vec3(" + a[0] + ", " + a[1] + ", " + a[2] + ")";
    }
    function exactEquals(a, b) {
      return a[0] === b[0] && a[1] === b[1] && a[2] === b[2];
    }
    function equals(a, b) {
      var a0 = a[0], a1 = a[1], a2 = a[2];
      var b0 = b[0], b1 = b[1], b2 = b[2];
      return Math.abs(a0 - b0) <= glMatrix.EPSILON * Math.max(1, Math.abs(a0), Math.abs(b0)) && Math.abs(a1 - b1) <= glMatrix.EPSILON * Math.max(1, Math.abs(a1), Math.abs(b1)) && Math.abs(a2 - b2) <= glMatrix.EPSILON * Math.max(1, Math.abs(a2), Math.abs(b2));
    }
    var sub = exports2.sub = subtract;
    var mul = exports2.mul = multiply;
    var div = exports2.div = divide;
    var dist = exports2.dist = distance;
    var sqrDist = exports2.sqrDist = squaredDistance;
    var len = exports2.len = length;
    var sqrLen = exports2.sqrLen = squaredLength;
    var forEach = exports2.forEach = (function() {
      var vec = create();
      return function(a, stride, offset, count, fn, arg) {
        var i, l;
        if (!stride) {
          stride = 3;
        }
        if (!offset) {
          offset = 0;
        }
        if (count) {
          l = Math.min(count * stride + offset, a.length);
        } else {
          l = a.length;
        }
        for (i = offset; i < l; i += stride) {
          vec[0] = a[i];
          vec[1] = a[i + 1];
          vec[2] = a[i + 2];
          fn(vec, vec, arg);
          a[i] = vec[0];
          a[i + 1] = vec[1];
          a[i + 2] = vec[2];
        }
        return a;
      };
    })();
  }
});

// {TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/vec4.js
var require_vec4 = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/vec4.js"(exports2) {
    "use strict";
    function _typeof(o) {
      "@babel/helpers - typeof";
      return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function(o2) {
        return typeof o2;
      } : function(o2) {
        return o2 && "function" == typeof Symbol && o2.constructor === Symbol && o2 !== Symbol.prototype ? "symbol" : typeof o2;
      }, _typeof(o);
    }
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.add = add;
    exports2.ceil = ceil;
    exports2.clone = clone;
    exports2.copy = copy;
    exports2.create = create;
    exports2.cross = cross;
    exports2.dist = void 0;
    exports2.distance = distance;
    exports2.div = void 0;
    exports2.divide = divide;
    exports2.dot = dot;
    exports2.equals = equals;
    exports2.exactEquals = exactEquals;
    exports2.floor = floor;
    exports2.forEach = void 0;
    exports2.fromValues = fromValues;
    exports2.inverse = inverse;
    exports2.len = void 0;
    exports2.length = length;
    exports2.lerp = lerp;
    exports2.max = max;
    exports2.min = min;
    exports2.mul = void 0;
    exports2.multiply = multiply;
    exports2.negate = negate;
    exports2.normalize = normalize;
    exports2.random = random;
    exports2.round = round;
    exports2.scale = scale;
    exports2.scaleAndAdd = scaleAndAdd;
    exports2.set = set;
    exports2.sqrLen = exports2.sqrDist = void 0;
    exports2.squaredDistance = squaredDistance;
    exports2.squaredLength = squaredLength;
    exports2.str = str;
    exports2.sub = void 0;
    exports2.subtract = subtract;
    exports2.transformMat4 = transformMat4;
    exports2.transformQuat = transformQuat;
    exports2.zero = zero;
    var glMatrix = _interopRequireWildcard(require_common());
    function _interopRequireWildcard(e, t) {
      if ("function" == typeof WeakMap) var r = /* @__PURE__ */ new WeakMap(), n = /* @__PURE__ */ new WeakMap();
      return (_interopRequireWildcard = function _interopRequireWildcard2(e2, t2) {
        if (!t2 && e2 && e2.__esModule) return e2;
        var o, i, f = { __proto__: null, "default": e2 };
        if (null === e2 || "object" != _typeof(e2) && "function" != typeof e2) return f;
        if (o = t2 ? n : r) {
          if (o.has(e2)) return o.get(e2);
          o.set(e2, f);
        }
        for (var _t in e2) "default" !== _t && {}.hasOwnProperty.call(e2, _t) && ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e2, _t)) && (i.get || i.set) ? o(f, _t, i) : f[_t] = e2[_t]);
        return f;
      })(e, t);
    }
    function create() {
      var out = new glMatrix.ARRAY_TYPE(4);
      if (glMatrix.ARRAY_TYPE != Float32Array) {
        out[0] = 0;
        out[1] = 0;
        out[2] = 0;
        out[3] = 0;
      }
      return out;
    }
    function clone(a) {
      var out = new glMatrix.ARRAY_TYPE(4);
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      out[3] = a[3];
      return out;
    }
    function fromValues(x, y, z, w) {
      var out = new glMatrix.ARRAY_TYPE(4);
      out[0] = x;
      out[1] = y;
      out[2] = z;
      out[3] = w;
      return out;
    }
    function copy(out, a) {
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      out[3] = a[3];
      return out;
    }
    function set(out, x, y, z, w) {
      out[0] = x;
      out[1] = y;
      out[2] = z;
      out[3] = w;
      return out;
    }
    function add(out, a, b) {
      out[0] = a[0] + b[0];
      out[1] = a[1] + b[1];
      out[2] = a[2] + b[2];
      out[3] = a[3] + b[3];
      return out;
    }
    function subtract(out, a, b) {
      out[0] = a[0] - b[0];
      out[1] = a[1] - b[1];
      out[2] = a[2] - b[2];
      out[3] = a[3] - b[3];
      return out;
    }
    function multiply(out, a, b) {
      out[0] = a[0] * b[0];
      out[1] = a[1] * b[1];
      out[2] = a[2] * b[2];
      out[3] = a[3] * b[3];
      return out;
    }
    function divide(out, a, b) {
      out[0] = a[0] / b[0];
      out[1] = a[1] / b[1];
      out[2] = a[2] / b[2];
      out[3] = a[3] / b[3];
      return out;
    }
    function ceil(out, a) {
      out[0] = Math.ceil(a[0]);
      out[1] = Math.ceil(a[1]);
      out[2] = Math.ceil(a[2]);
      out[3] = Math.ceil(a[3]);
      return out;
    }
    function floor(out, a) {
      out[0] = Math.floor(a[0]);
      out[1] = Math.floor(a[1]);
      out[2] = Math.floor(a[2]);
      out[3] = Math.floor(a[3]);
      return out;
    }
    function min(out, a, b) {
      out[0] = Math.min(a[0], b[0]);
      out[1] = Math.min(a[1], b[1]);
      out[2] = Math.min(a[2], b[2]);
      out[3] = Math.min(a[3], b[3]);
      return out;
    }
    function max(out, a, b) {
      out[0] = Math.max(a[0], b[0]);
      out[1] = Math.max(a[1], b[1]);
      out[2] = Math.max(a[2], b[2]);
      out[3] = Math.max(a[3], b[3]);
      return out;
    }
    function round(out, a) {
      out[0] = glMatrix.round(a[0]);
      out[1] = glMatrix.round(a[1]);
      out[2] = glMatrix.round(a[2]);
      out[3] = glMatrix.round(a[3]);
      return out;
    }
    function scale(out, a, b) {
      out[0] = a[0] * b;
      out[1] = a[1] * b;
      out[2] = a[2] * b;
      out[3] = a[3] * b;
      return out;
    }
    function scaleAndAdd(out, a, b, scale2) {
      out[0] = a[0] + b[0] * scale2;
      out[1] = a[1] + b[1] * scale2;
      out[2] = a[2] + b[2] * scale2;
      out[3] = a[3] + b[3] * scale2;
      return out;
    }
    function distance(a, b) {
      var x = b[0] - a[0];
      var y = b[1] - a[1];
      var z = b[2] - a[2];
      var w = b[3] - a[3];
      return Math.sqrt(x * x + y * y + z * z + w * w);
    }
    function squaredDistance(a, b) {
      var x = b[0] - a[0];
      var y = b[1] - a[1];
      var z = b[2] - a[2];
      var w = b[3] - a[3];
      return x * x + y * y + z * z + w * w;
    }
    function length(a) {
      var x = a[0];
      var y = a[1];
      var z = a[2];
      var w = a[3];
      return Math.sqrt(x * x + y * y + z * z + w * w);
    }
    function squaredLength(a) {
      var x = a[0];
      var y = a[1];
      var z = a[2];
      var w = a[3];
      return x * x + y * y + z * z + w * w;
    }
    function negate(out, a) {
      out[0] = -a[0];
      out[1] = -a[1];
      out[2] = -a[2];
      out[3] = -a[3];
      return out;
    }
    function inverse(out, a) {
      out[0] = 1 / a[0];
      out[1] = 1 / a[1];
      out[2] = 1 / a[2];
      out[3] = 1 / a[3];
      return out;
    }
    function normalize(out, a) {
      var x = a[0];
      var y = a[1];
      var z = a[2];
      var w = a[3];
      var len2 = x * x + y * y + z * z + w * w;
      if (len2 > 0) {
        len2 = 1 / Math.sqrt(len2);
      }
      out[0] = x * len2;
      out[1] = y * len2;
      out[2] = z * len2;
      out[3] = w * len2;
      return out;
    }
    function dot(a, b) {
      return a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    }
    function cross(out, u, v, w) {
      var A = v[0] * w[1] - v[1] * w[0], B = v[0] * w[2] - v[2] * w[0], C = v[0] * w[3] - v[3] * w[0], D = v[1] * w[2] - v[2] * w[1], E = v[1] * w[3] - v[3] * w[1], F = v[2] * w[3] - v[3] * w[2];
      var G = u[0];
      var H = u[1];
      var I = u[2];
      var J = u[3];
      out[0] = H * F - I * E + J * D;
      out[1] = -(G * F) + I * C - J * B;
      out[2] = G * E - H * C + J * A;
      out[3] = -(G * D) + H * B - I * A;
      return out;
    }
    function lerp(out, a, b, t) {
      var ax = a[0];
      var ay = a[1];
      var az = a[2];
      var aw = a[3];
      out[0] = ax + t * (b[0] - ax);
      out[1] = ay + t * (b[1] - ay);
      out[2] = az + t * (b[2] - az);
      out[3] = aw + t * (b[3] - aw);
      return out;
    }
    function random(out, scale2) {
      scale2 = scale2 === void 0 ? 1 : scale2;
      var v1, v2, v3, v4;
      var s1, s2;
      var rand;
      rand = glMatrix.RANDOM();
      v1 = rand * 2 - 1;
      v2 = (4 * glMatrix.RANDOM() - 2) * Math.sqrt(rand * -rand + rand);
      s1 = v1 * v1 + v2 * v2;
      rand = glMatrix.RANDOM();
      v3 = rand * 2 - 1;
      v4 = (4 * glMatrix.RANDOM() - 2) * Math.sqrt(rand * -rand + rand);
      s2 = v3 * v3 + v4 * v4;
      var d2 = Math.sqrt((1 - s1) / s2);
      out[0] = scale2 * v1;
      out[1] = scale2 * v2;
      out[2] = scale2 * v3 * d2;
      out[3] = scale2 * v4 * d2;
      return out;
    }
    function transformMat4(out, a, m) {
      var x = a[0], y = a[1], z = a[2], w = a[3];
      out[0] = m[0] * x + m[4] * y + m[8] * z + m[12] * w;
      out[1] = m[1] * x + m[5] * y + m[9] * z + m[13] * w;
      out[2] = m[2] * x + m[6] * y + m[10] * z + m[14] * w;
      out[3] = m[3] * x + m[7] * y + m[11] * z + m[15] * w;
      return out;
    }
    function transformQuat(out, a, q) {
      var qx = q[0], qy = q[1], qz = q[2], qw = q[3];
      var vx = a[0], vy = a[1], vz = a[2];
      var tx = qy * vz - qz * vy;
      var ty = qz * vx - qx * vz;
      var tz = qx * vy - qy * vx;
      tx = tx + tx;
      ty = ty + ty;
      tz = tz + tz;
      out[0] = vx + qw * tx + qy * tz - qz * ty;
      out[1] = vy + qw * ty + qz * tx - qx * tz;
      out[2] = vz + qw * tz + qx * ty - qy * tx;
      out[3] = a[3];
      return out;
    }
    function zero(out) {
      out[0] = 0;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      return out;
    }
    function str(a) {
      return "vec4(" + a[0] + ", " + a[1] + ", " + a[2] + ", " + a[3] + ")";
    }
    function exactEquals(a, b) {
      return a[0] === b[0] && a[1] === b[1] && a[2] === b[2] && a[3] === b[3];
    }
    function equals(a, b) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3];
      var b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3];
      return Math.abs(a0 - b0) <= glMatrix.EPSILON * Math.max(1, Math.abs(a0), Math.abs(b0)) && Math.abs(a1 - b1) <= glMatrix.EPSILON * Math.max(1, Math.abs(a1), Math.abs(b1)) && Math.abs(a2 - b2) <= glMatrix.EPSILON * Math.max(1, Math.abs(a2), Math.abs(b2)) && Math.abs(a3 - b3) <= glMatrix.EPSILON * Math.max(1, Math.abs(a3), Math.abs(b3));
    }
    var sub = exports2.sub = subtract;
    var mul = exports2.mul = multiply;
    var div = exports2.div = divide;
    var dist = exports2.dist = distance;
    var sqrDist = exports2.sqrDist = squaredDistance;
    var len = exports2.len = length;
    var sqrLen = exports2.sqrLen = squaredLength;
    var forEach = exports2.forEach = (function() {
      var vec = create();
      return function(a, stride, offset, count, fn, arg) {
        var i, l;
        if (!stride) {
          stride = 4;
        }
        if (!offset) {
          offset = 0;
        }
        if (count) {
          l = Math.min(count * stride + offset, a.length);
        } else {
          l = a.length;
        }
        for (i = offset; i < l; i += stride) {
          vec[0] = a[i];
          vec[1] = a[i + 1];
          vec[2] = a[i + 2];
          vec[3] = a[i + 3];
          fn(vec, vec, arg);
          a[i] = vec[0];
          a[i + 1] = vec[1];
          a[i + 2] = vec[2];
          a[i + 3] = vec[3];
        }
        return a;
      };
    })();
  }
});

// {TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/quat.js
var require_quat = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/quat.js"(exports2) {
    "use strict";
    function _typeof(o) {
      "@babel/helpers - typeof";
      return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function(o2) {
        return typeof o2;
      } : function(o2) {
        return o2 && "function" == typeof Symbol && o2.constructor === Symbol && o2 !== Symbol.prototype ? "symbol" : typeof o2;
      }, _typeof(o);
    }
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.add = void 0;
    exports2.calculateW = calculateW;
    exports2.clone = void 0;
    exports2.conjugate = conjugate;
    exports2.copy = void 0;
    exports2.create = create;
    exports2.dot = void 0;
    exports2.equals = equals;
    exports2.exactEquals = void 0;
    exports2.exp = exp;
    exports2.fromEuler = fromEuler;
    exports2.fromMat3 = fromMat3;
    exports2.fromValues = void 0;
    exports2.getAngle = getAngle;
    exports2.getAxisAngle = getAxisAngle;
    exports2.identity = identity;
    exports2.invert = invert;
    exports2.lerp = exports2.length = exports2.len = void 0;
    exports2.ln = ln;
    exports2.mul = void 0;
    exports2.multiply = multiply;
    exports2.normalize = void 0;
    exports2.pow = pow;
    exports2.random = random;
    exports2.rotateX = rotateX;
    exports2.rotateY = rotateY;
    exports2.rotateZ = rotateZ;
    exports2.setAxes = exports2.set = exports2.scale = exports2.rotationTo = void 0;
    exports2.setAxisAngle = setAxisAngle;
    exports2.slerp = slerp;
    exports2.squaredLength = exports2.sqrLen = exports2.sqlerp = void 0;
    exports2.str = str;
    var glMatrix = _interopRequireWildcard(require_common());
    var mat3 = _interopRequireWildcard(require_mat3());
    var vec32 = _interopRequireWildcard(require_vec3());
    var vec4 = _interopRequireWildcard(require_vec4());
    function _interopRequireWildcard(e, t) {
      if ("function" == typeof WeakMap) var r = /* @__PURE__ */ new WeakMap(), n = /* @__PURE__ */ new WeakMap();
      return (_interopRequireWildcard = function _interopRequireWildcard2(e2, t2) {
        if (!t2 && e2 && e2.__esModule) return e2;
        var o, i, f = { __proto__: null, "default": e2 };
        if (null === e2 || "object" != _typeof(e2) && "function" != typeof e2) return f;
        if (o = t2 ? n : r) {
          if (o.has(e2)) return o.get(e2);
          o.set(e2, f);
        }
        for (var _t in e2) "default" !== _t && {}.hasOwnProperty.call(e2, _t) && ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e2, _t)) && (i.get || i.set) ? o(f, _t, i) : f[_t] = e2[_t]);
        return f;
      })(e, t);
    }
    function create() {
      var out = new glMatrix.ARRAY_TYPE(4);
      if (glMatrix.ARRAY_TYPE != Float32Array) {
        out[0] = 0;
        out[1] = 0;
        out[2] = 0;
      }
      out[3] = 1;
      return out;
    }
    function identity(out) {
      out[0] = 0;
      out[1] = 0;
      out[2] = 0;
      out[3] = 1;
      return out;
    }
    function setAxisAngle(out, axis, rad) {
      rad = rad * 0.5;
      var s = Math.sin(rad);
      out[0] = s * axis[0];
      out[1] = s * axis[1];
      out[2] = s * axis[2];
      out[3] = Math.cos(rad);
      return out;
    }
    function getAxisAngle(out_axis, q) {
      var rad = Math.acos(q[3]) * 2;
      var s = Math.sin(rad / 2);
      if (s > glMatrix.EPSILON) {
        out_axis[0] = q[0] / s;
        out_axis[1] = q[1] / s;
        out_axis[2] = q[2] / s;
      } else {
        out_axis[0] = 1;
        out_axis[1] = 0;
        out_axis[2] = 0;
      }
      return rad;
    }
    function getAngle(a, b) {
      var dotproduct = dot(a, b);
      return Math.acos(2 * dotproduct * dotproduct - 1);
    }
    function multiply(out, a, b) {
      var ax = a[0], ay = a[1], az = a[2], aw = a[3];
      var bx = b[0], by = b[1], bz = b[2], bw = b[3];
      out[0] = ax * bw + aw * bx + ay * bz - az * by;
      out[1] = ay * bw + aw * by + az * bx - ax * bz;
      out[2] = az * bw + aw * bz + ax * by - ay * bx;
      out[3] = aw * bw - ax * bx - ay * by - az * bz;
      return out;
    }
    function rotateX(out, a, rad) {
      rad *= 0.5;
      var ax = a[0], ay = a[1], az = a[2], aw = a[3];
      var bx = Math.sin(rad), bw = Math.cos(rad);
      out[0] = ax * bw + aw * bx;
      out[1] = ay * bw + az * bx;
      out[2] = az * bw - ay * bx;
      out[3] = aw * bw - ax * bx;
      return out;
    }
    function rotateY(out, a, rad) {
      rad *= 0.5;
      var ax = a[0], ay = a[1], az = a[2], aw = a[3];
      var by = Math.sin(rad), bw = Math.cos(rad);
      out[0] = ax * bw - az * by;
      out[1] = ay * bw + aw * by;
      out[2] = az * bw + ax * by;
      out[3] = aw * bw - ay * by;
      return out;
    }
    function rotateZ(out, a, rad) {
      rad *= 0.5;
      var ax = a[0], ay = a[1], az = a[2], aw = a[3];
      var bz = Math.sin(rad), bw = Math.cos(rad);
      out[0] = ax * bw + ay * bz;
      out[1] = ay * bw - ax * bz;
      out[2] = az * bw + aw * bz;
      out[3] = aw * bw - az * bz;
      return out;
    }
    function calculateW(out, a) {
      var x = a[0], y = a[1], z = a[2];
      out[0] = x;
      out[1] = y;
      out[2] = z;
      out[3] = Math.sqrt(Math.abs(1 - x * x - y * y - z * z));
      return out;
    }
    function exp(out, a) {
      var x = a[0], y = a[1], z = a[2], w = a[3];
      var r = Math.sqrt(x * x + y * y + z * z);
      var et = Math.exp(w);
      var s = r > 0 ? et * Math.sin(r) / r : 0;
      out[0] = x * s;
      out[1] = y * s;
      out[2] = z * s;
      out[3] = et * Math.cos(r);
      return out;
    }
    function ln(out, a) {
      var x = a[0], y = a[1], z = a[2], w = a[3];
      var r = Math.sqrt(x * x + y * y + z * z);
      var t = r > 0 ? Math.atan2(r, w) / r : 0;
      out[0] = x * t;
      out[1] = y * t;
      out[2] = z * t;
      out[3] = 0.5 * Math.log(x * x + y * y + z * z + w * w);
      return out;
    }
    function pow(out, a, b) {
      ln(out, a);
      scale(out, out, b);
      exp(out, out);
      return out;
    }
    function slerp(out, a, b, t) {
      var ax = a[0], ay = a[1], az = a[2], aw = a[3];
      var bx = b[0], by = b[1], bz = b[2], bw = b[3];
      var omega, cosom, sinom, scale0, scale1;
      cosom = ax * bx + ay * by + az * bz + aw * bw;
      if (cosom < 0) {
        cosom = -cosom;
        bx = -bx;
        by = -by;
        bz = -bz;
        bw = -bw;
      }
      if (1 - cosom > glMatrix.EPSILON) {
        omega = Math.acos(cosom);
        sinom = Math.sin(omega);
        scale0 = Math.sin((1 - t) * omega) / sinom;
        scale1 = Math.sin(t * omega) / sinom;
      } else {
        scale0 = 1 - t;
        scale1 = t;
      }
      out[0] = scale0 * ax + scale1 * bx;
      out[1] = scale0 * ay + scale1 * by;
      out[2] = scale0 * az + scale1 * bz;
      out[3] = scale0 * aw + scale1 * bw;
      return out;
    }
    function random(out) {
      var u1 = glMatrix.RANDOM();
      var u2 = glMatrix.RANDOM();
      var u3 = glMatrix.RANDOM();
      var sqrt1MinusU1 = Math.sqrt(1 - u1);
      var sqrtU1 = Math.sqrt(u1);
      out[0] = sqrt1MinusU1 * Math.sin(2 * Math.PI * u2);
      out[1] = sqrt1MinusU1 * Math.cos(2 * Math.PI * u2);
      out[2] = sqrtU1 * Math.sin(2 * Math.PI * u3);
      out[3] = sqrtU1 * Math.cos(2 * Math.PI * u3);
      return out;
    }
    function invert(out, a) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3];
      var dot2 = a0 * a0 + a1 * a1 + a2 * a2 + a3 * a3;
      var invDot = dot2 ? 1 / dot2 : 0;
      out[0] = -a0 * invDot;
      out[1] = -a1 * invDot;
      out[2] = -a2 * invDot;
      out[3] = a3 * invDot;
      return out;
    }
    function conjugate(out, a) {
      out[0] = -a[0];
      out[1] = -a[1];
      out[2] = -a[2];
      out[3] = a[3];
      return out;
    }
    function fromMat3(out, m) {
      var fTrace = m[0] + m[4] + m[8];
      var fRoot;
      if (fTrace > 0) {
        fRoot = Math.sqrt(fTrace + 1);
        out[3] = 0.5 * fRoot;
        fRoot = 0.5 / fRoot;
        out[0] = (m[5] - m[7]) * fRoot;
        out[1] = (m[6] - m[2]) * fRoot;
        out[2] = (m[1] - m[3]) * fRoot;
      } else {
        var i = 0;
        if (m[4] > m[0]) i = 1;
        if (m[8] > m[i * 3 + i]) i = 2;
        var j = (i + 1) % 3;
        var k = (i + 2) % 3;
        fRoot = Math.sqrt(m[i * 3 + i] - m[j * 3 + j] - m[k * 3 + k] + 1);
        out[i] = 0.5 * fRoot;
        fRoot = 0.5 / fRoot;
        out[3] = (m[j * 3 + k] - m[k * 3 + j]) * fRoot;
        out[j] = (m[j * 3 + i] + m[i * 3 + j]) * fRoot;
        out[k] = (m[k * 3 + i] + m[i * 3 + k]) * fRoot;
      }
      return out;
    }
    function fromEuler(out, x, y, z) {
      var order = arguments.length > 4 && arguments[4] !== void 0 ? arguments[4] : glMatrix.ANGLE_ORDER;
      var halfToRad = Math.PI / 360;
      x *= halfToRad;
      z *= halfToRad;
      y *= halfToRad;
      var sx = Math.sin(x);
      var cx = Math.cos(x);
      var sy = Math.sin(y);
      var cy = Math.cos(y);
      var sz = Math.sin(z);
      var cz = Math.cos(z);
      switch (order) {
        case "xyz":
          out[0] = sx * cy * cz + cx * sy * sz;
          out[1] = cx * sy * cz - sx * cy * sz;
          out[2] = cx * cy * sz + sx * sy * cz;
          out[3] = cx * cy * cz - sx * sy * sz;
          break;
        case "xzy":
          out[0] = sx * cy * cz - cx * sy * sz;
          out[1] = cx * sy * cz - sx * cy * sz;
          out[2] = cx * cy * sz + sx * sy * cz;
          out[3] = cx * cy * cz + sx * sy * sz;
          break;
        case "yxz":
          out[0] = sx * cy * cz + cx * sy * sz;
          out[1] = cx * sy * cz - sx * cy * sz;
          out[2] = cx * cy * sz - sx * sy * cz;
          out[3] = cx * cy * cz + sx * sy * sz;
          break;
        case "yzx":
          out[0] = sx * cy * cz + cx * sy * sz;
          out[1] = cx * sy * cz + sx * cy * sz;
          out[2] = cx * cy * sz - sx * sy * cz;
          out[3] = cx * cy * cz - sx * sy * sz;
          break;
        case "zxy":
          out[0] = sx * cy * cz - cx * sy * sz;
          out[1] = cx * sy * cz + sx * cy * sz;
          out[2] = cx * cy * sz + sx * sy * cz;
          out[3] = cx * cy * cz - sx * sy * sz;
          break;
        case "zyx":
          out[0] = sx * cy * cz - cx * sy * sz;
          out[1] = cx * sy * cz + sx * cy * sz;
          out[2] = cx * cy * sz - sx * sy * cz;
          out[3] = cx * cy * cz + sx * sy * sz;
          break;
        default:
          throw new Error("Unknown angle order " + order);
      }
      return out;
    }
    function str(a) {
      return "quat(" + a[0] + ", " + a[1] + ", " + a[2] + ", " + a[3] + ")";
    }
    var clone = exports2.clone = vec4.clone;
    var fromValues = exports2.fromValues = vec4.fromValues;
    var copy = exports2.copy = vec4.copy;
    var set = exports2.set = vec4.set;
    var add = exports2.add = vec4.add;
    var mul = exports2.mul = multiply;
    var scale = exports2.scale = vec4.scale;
    var dot = exports2.dot = vec4.dot;
    var lerp = exports2.lerp = vec4.lerp;
    var length = exports2.length = vec4.length;
    var len = exports2.len = length;
    var squaredLength = exports2.squaredLength = vec4.squaredLength;
    var sqrLen = exports2.sqrLen = squaredLength;
    var normalize = exports2.normalize = vec4.normalize;
    var exactEquals = exports2.exactEquals = vec4.exactEquals;
    function equals(a, b) {
      return Math.abs(vec4.dot(a, b)) >= 1 - glMatrix.EPSILON;
    }
    var rotationTo = exports2.rotationTo = (function() {
      var tmpvec3 = vec32.create();
      var xUnitVec3 = vec32.fromValues(1, 0, 0);
      var yUnitVec3 = vec32.fromValues(0, 1, 0);
      return function(out, a, b) {
        var dot2 = vec32.dot(a, b);
        if (dot2 < -0.999999) {
          vec32.cross(tmpvec3, xUnitVec3, a);
          if (vec32.len(tmpvec3) < 1e-6) vec32.cross(tmpvec3, yUnitVec3, a);
          vec32.normalize(tmpvec3, tmpvec3);
          setAxisAngle(out, tmpvec3, Math.PI);
          return out;
        } else if (dot2 > 0.999999) {
          out[0] = 0;
          out[1] = 0;
          out[2] = 0;
          out[3] = 1;
          return out;
        } else {
          vec32.cross(tmpvec3, a, b);
          out[0] = tmpvec3[0];
          out[1] = tmpvec3[1];
          out[2] = tmpvec3[2];
          out[3] = 1 + dot2;
          return normalize(out, out);
        }
      };
    })();
    var sqlerp = exports2.sqlerp = (function() {
      var temp1 = create();
      var temp2 = create();
      return function(out, a, b, c, d2, t) {
        slerp(temp1, a, d2, t);
        slerp(temp2, b, c, t);
        slerp(out, temp1, temp2, 2 * t * (1 - t));
        return out;
      };
    })();
    var setAxes = exports2.setAxes = (function() {
      var matr = mat3.create();
      return function(out, view, right, up) {
        matr[0] = right[0];
        matr[3] = right[1];
        matr[6] = right[2];
        matr[1] = up[0];
        matr[4] = up[1];
        matr[7] = up[2];
        matr[2] = -view[0];
        matr[5] = -view[1];
        matr[8] = -view[2];
        return normalize(out, fromMat3(out, matr));
      };
    })();
  }
});

// {TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/quat2.js
var require_quat2 = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/quat2.js"(exports2) {
    "use strict";
    function _typeof(o) {
      "@babel/helpers - typeof";
      return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function(o2) {
        return typeof o2;
      } : function(o2) {
        return o2 && "function" == typeof Symbol && o2.constructor === Symbol && o2 !== Symbol.prototype ? "symbol" : typeof o2;
      }, _typeof(o);
    }
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.add = add;
    exports2.clone = clone;
    exports2.conjugate = conjugate;
    exports2.copy = copy;
    exports2.create = create;
    exports2.dot = void 0;
    exports2.equals = equals;
    exports2.exactEquals = exactEquals;
    exports2.fromMat4 = fromMat4;
    exports2.fromRotation = fromRotation;
    exports2.fromRotationTranslation = fromRotationTranslation;
    exports2.fromRotationTranslationValues = fromRotationTranslationValues;
    exports2.fromTranslation = fromTranslation;
    exports2.fromValues = fromValues;
    exports2.getDual = getDual;
    exports2.getReal = void 0;
    exports2.getTranslation = getTranslation;
    exports2.identity = identity;
    exports2.invert = invert;
    exports2.length = exports2.len = void 0;
    exports2.lerp = lerp;
    exports2.mul = void 0;
    exports2.multiply = multiply;
    exports2.normalize = normalize;
    exports2.rotateAroundAxis = rotateAroundAxis;
    exports2.rotateByQuatAppend = rotateByQuatAppend;
    exports2.rotateByQuatPrepend = rotateByQuatPrepend;
    exports2.rotateX = rotateX;
    exports2.rotateY = rotateY;
    exports2.rotateZ = rotateZ;
    exports2.scale = scale;
    exports2.set = set;
    exports2.setDual = setDual;
    exports2.squaredLength = exports2.sqrLen = exports2.setReal = void 0;
    exports2.str = str;
    exports2.translate = translate;
    var glMatrix = _interopRequireWildcard(require_common());
    var quat = _interopRequireWildcard(require_quat());
    var mat42 = _interopRequireWildcard(require_mat4());
    function _interopRequireWildcard(e, t) {
      if ("function" == typeof WeakMap) var r = /* @__PURE__ */ new WeakMap(), n = /* @__PURE__ */ new WeakMap();
      return (_interopRequireWildcard = function _interopRequireWildcard2(e2, t2) {
        if (!t2 && e2 && e2.__esModule) return e2;
        var o, i, f = { __proto__: null, "default": e2 };
        if (null === e2 || "object" != _typeof(e2) && "function" != typeof e2) return f;
        if (o = t2 ? n : r) {
          if (o.has(e2)) return o.get(e2);
          o.set(e2, f);
        }
        for (var _t in e2) "default" !== _t && {}.hasOwnProperty.call(e2, _t) && ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e2, _t)) && (i.get || i.set) ? o(f, _t, i) : f[_t] = e2[_t]);
        return f;
      })(e, t);
    }
    function create() {
      var dq = new glMatrix.ARRAY_TYPE(8);
      if (glMatrix.ARRAY_TYPE != Float32Array) {
        dq[0] = 0;
        dq[1] = 0;
        dq[2] = 0;
        dq[4] = 0;
        dq[5] = 0;
        dq[6] = 0;
        dq[7] = 0;
      }
      dq[3] = 1;
      return dq;
    }
    function clone(a) {
      var dq = new glMatrix.ARRAY_TYPE(8);
      dq[0] = a[0];
      dq[1] = a[1];
      dq[2] = a[2];
      dq[3] = a[3];
      dq[4] = a[4];
      dq[5] = a[5];
      dq[6] = a[6];
      dq[7] = a[7];
      return dq;
    }
    function fromValues(x1, y1, z1, w1, x2, y2, z2, w2) {
      var dq = new glMatrix.ARRAY_TYPE(8);
      dq[0] = x1;
      dq[1] = y1;
      dq[2] = z1;
      dq[3] = w1;
      dq[4] = x2;
      dq[5] = y2;
      dq[6] = z2;
      dq[7] = w2;
      return dq;
    }
    function fromRotationTranslationValues(x1, y1, z1, w1, x2, y2, z2) {
      var dq = new glMatrix.ARRAY_TYPE(8);
      dq[0] = x1;
      dq[1] = y1;
      dq[2] = z1;
      dq[3] = w1;
      var ax = x2 * 0.5, ay = y2 * 0.5, az = z2 * 0.5;
      dq[4] = ax * w1 + ay * z1 - az * y1;
      dq[5] = ay * w1 + az * x1 - ax * z1;
      dq[6] = az * w1 + ax * y1 - ay * x1;
      dq[7] = -ax * x1 - ay * y1 - az * z1;
      return dq;
    }
    function fromRotationTranslation(out, q, t) {
      var ax = t[0] * 0.5, ay = t[1] * 0.5, az = t[2] * 0.5, bx = q[0], by = q[1], bz = q[2], bw = q[3];
      out[0] = bx;
      out[1] = by;
      out[2] = bz;
      out[3] = bw;
      out[4] = ax * bw + ay * bz - az * by;
      out[5] = ay * bw + az * bx - ax * bz;
      out[6] = az * bw + ax * by - ay * bx;
      out[7] = -ax * bx - ay * by - az * bz;
      return out;
    }
    function fromTranslation(out, t) {
      out[0] = 0;
      out[1] = 0;
      out[2] = 0;
      out[3] = 1;
      out[4] = t[0] * 0.5;
      out[5] = t[1] * 0.5;
      out[6] = t[2] * 0.5;
      out[7] = 0;
      return out;
    }
    function fromRotation(out, q) {
      out[0] = q[0];
      out[1] = q[1];
      out[2] = q[2];
      out[3] = q[3];
      out[4] = 0;
      out[5] = 0;
      out[6] = 0;
      out[7] = 0;
      return out;
    }
    function fromMat4(out, a) {
      var outer = quat.create();
      mat42.getRotation(outer, a);
      var t = new glMatrix.ARRAY_TYPE(3);
      mat42.getTranslation(t, a);
      fromRotationTranslation(out, outer, t);
      return out;
    }
    function copy(out, a) {
      out[0] = a[0];
      out[1] = a[1];
      out[2] = a[2];
      out[3] = a[3];
      out[4] = a[4];
      out[5] = a[5];
      out[6] = a[6];
      out[7] = a[7];
      return out;
    }
    function identity(out) {
      out[0] = 0;
      out[1] = 0;
      out[2] = 0;
      out[3] = 1;
      out[4] = 0;
      out[5] = 0;
      out[6] = 0;
      out[7] = 0;
      return out;
    }
    function set(out, x1, y1, z1, w1, x2, y2, z2, w2) {
      out[0] = x1;
      out[1] = y1;
      out[2] = z1;
      out[3] = w1;
      out[4] = x2;
      out[5] = y2;
      out[6] = z2;
      out[7] = w2;
      return out;
    }
    var getReal = exports2.getReal = quat.copy;
    function getDual(out, a) {
      out[0] = a[4];
      out[1] = a[5];
      out[2] = a[6];
      out[3] = a[7];
      return out;
    }
    var setReal = exports2.setReal = quat.copy;
    function setDual(out, q) {
      out[4] = q[0];
      out[5] = q[1];
      out[6] = q[2];
      out[7] = q[3];
      return out;
    }
    function getTranslation(out, a) {
      var ax = a[4], ay = a[5], az = a[6], aw = a[7], bx = -a[0], by = -a[1], bz = -a[2], bw = a[3];
      out[0] = (ax * bw + aw * bx + ay * bz - az * by) * 2;
      out[1] = (ay * bw + aw * by + az * bx - ax * bz) * 2;
      out[2] = (az * bw + aw * bz + ax * by - ay * bx) * 2;
      return out;
    }
    function translate(out, a, v) {
      var ax1 = a[0], ay1 = a[1], az1 = a[2], aw1 = a[3], bx1 = v[0] * 0.5, by1 = v[1] * 0.5, bz1 = v[2] * 0.5, ax2 = a[4], ay2 = a[5], az2 = a[6], aw2 = a[7];
      out[0] = ax1;
      out[1] = ay1;
      out[2] = az1;
      out[3] = aw1;
      out[4] = aw1 * bx1 + ay1 * bz1 - az1 * by1 + ax2;
      out[5] = aw1 * by1 + az1 * bx1 - ax1 * bz1 + ay2;
      out[6] = aw1 * bz1 + ax1 * by1 - ay1 * bx1 + az2;
      out[7] = -ax1 * bx1 - ay1 * by1 - az1 * bz1 + aw2;
      return out;
    }
    function rotateX(out, a, rad) {
      var bx = -a[0], by = -a[1], bz = -a[2], bw = a[3], ax = a[4], ay = a[5], az = a[6], aw = a[7], ax1 = ax * bw + aw * bx + ay * bz - az * by, ay1 = ay * bw + aw * by + az * bx - ax * bz, az1 = az * bw + aw * bz + ax * by - ay * bx, aw1 = aw * bw - ax * bx - ay * by - az * bz;
      quat.rotateX(out, a, rad);
      bx = out[0];
      by = out[1];
      bz = out[2];
      bw = out[3];
      out[4] = ax1 * bw + aw1 * bx + ay1 * bz - az1 * by;
      out[5] = ay1 * bw + aw1 * by + az1 * bx - ax1 * bz;
      out[6] = az1 * bw + aw1 * bz + ax1 * by - ay1 * bx;
      out[7] = aw1 * bw - ax1 * bx - ay1 * by - az1 * bz;
      return out;
    }
    function rotateY(out, a, rad) {
      var bx = -a[0], by = -a[1], bz = -a[2], bw = a[3], ax = a[4], ay = a[5], az = a[6], aw = a[7], ax1 = ax * bw + aw * bx + ay * bz - az * by, ay1 = ay * bw + aw * by + az * bx - ax * bz, az1 = az * bw + aw * bz + ax * by - ay * bx, aw1 = aw * bw - ax * bx - ay * by - az * bz;
      quat.rotateY(out, a, rad);
      bx = out[0];
      by = out[1];
      bz = out[2];
      bw = out[3];
      out[4] = ax1 * bw + aw1 * bx + ay1 * bz - az1 * by;
      out[5] = ay1 * bw + aw1 * by + az1 * bx - ax1 * bz;
      out[6] = az1 * bw + aw1 * bz + ax1 * by - ay1 * bx;
      out[7] = aw1 * bw - ax1 * bx - ay1 * by - az1 * bz;
      return out;
    }
    function rotateZ(out, a, rad) {
      var bx = -a[0], by = -a[1], bz = -a[2], bw = a[3], ax = a[4], ay = a[5], az = a[6], aw = a[7], ax1 = ax * bw + aw * bx + ay * bz - az * by, ay1 = ay * bw + aw * by + az * bx - ax * bz, az1 = az * bw + aw * bz + ax * by - ay * bx, aw1 = aw * bw - ax * bx - ay * by - az * bz;
      quat.rotateZ(out, a, rad);
      bx = out[0];
      by = out[1];
      bz = out[2];
      bw = out[3];
      out[4] = ax1 * bw + aw1 * bx + ay1 * bz - az1 * by;
      out[5] = ay1 * bw + aw1 * by + az1 * bx - ax1 * bz;
      out[6] = az1 * bw + aw1 * bz + ax1 * by - ay1 * bx;
      out[7] = aw1 * bw - ax1 * bx - ay1 * by - az1 * bz;
      return out;
    }
    function rotateByQuatAppend(out, a, q) {
      var qx = q[0], qy = q[1], qz = q[2], qw = q[3], ax = a[0], ay = a[1], az = a[2], aw = a[3];
      out[0] = ax * qw + aw * qx + ay * qz - az * qy;
      out[1] = ay * qw + aw * qy + az * qx - ax * qz;
      out[2] = az * qw + aw * qz + ax * qy - ay * qx;
      out[3] = aw * qw - ax * qx - ay * qy - az * qz;
      ax = a[4];
      ay = a[5];
      az = a[6];
      aw = a[7];
      out[4] = ax * qw + aw * qx + ay * qz - az * qy;
      out[5] = ay * qw + aw * qy + az * qx - ax * qz;
      out[6] = az * qw + aw * qz + ax * qy - ay * qx;
      out[7] = aw * qw - ax * qx - ay * qy - az * qz;
      return out;
    }
    function rotateByQuatPrepend(out, q, a) {
      var qx = q[0], qy = q[1], qz = q[2], qw = q[3], bx = a[0], by = a[1], bz = a[2], bw = a[3];
      out[0] = qx * bw + qw * bx + qy * bz - qz * by;
      out[1] = qy * bw + qw * by + qz * bx - qx * bz;
      out[2] = qz * bw + qw * bz + qx * by - qy * bx;
      out[3] = qw * bw - qx * bx - qy * by - qz * bz;
      bx = a[4];
      by = a[5];
      bz = a[6];
      bw = a[7];
      out[4] = qx * bw + qw * bx + qy * bz - qz * by;
      out[5] = qy * bw + qw * by + qz * bx - qx * bz;
      out[6] = qz * bw + qw * bz + qx * by - qy * bx;
      out[7] = qw * bw - qx * bx - qy * by - qz * bz;
      return out;
    }
    function rotateAroundAxis(out, a, axis, rad) {
      if (Math.abs(rad) < glMatrix.EPSILON) {
        return copy(out, a);
      }
      var axisLength = Math.sqrt(axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]);
      rad = rad * 0.5;
      var s = Math.sin(rad);
      var bx = s * axis[0] / axisLength;
      var by = s * axis[1] / axisLength;
      var bz = s * axis[2] / axisLength;
      var bw = Math.cos(rad);
      var ax1 = a[0], ay1 = a[1], az1 = a[2], aw1 = a[3];
      out[0] = ax1 * bw + aw1 * bx + ay1 * bz - az1 * by;
      out[1] = ay1 * bw + aw1 * by + az1 * bx - ax1 * bz;
      out[2] = az1 * bw + aw1 * bz + ax1 * by - ay1 * bx;
      out[3] = aw1 * bw - ax1 * bx - ay1 * by - az1 * bz;
      var ax = a[4], ay = a[5], az = a[6], aw = a[7];
      out[4] = ax * bw + aw * bx + ay * bz - az * by;
      out[5] = ay * bw + aw * by + az * bx - ax * bz;
      out[6] = az * bw + aw * bz + ax * by - ay * bx;
      out[7] = aw * bw - ax * bx - ay * by - az * bz;
      return out;
    }
    function add(out, a, b) {
      out[0] = a[0] + b[0];
      out[1] = a[1] + b[1];
      out[2] = a[2] + b[2];
      out[3] = a[3] + b[3];
      out[4] = a[4] + b[4];
      out[5] = a[5] + b[5];
      out[6] = a[6] + b[6];
      out[7] = a[7] + b[7];
      return out;
    }
    function multiply(out, a, b) {
      var ax0 = a[0], ay0 = a[1], az0 = a[2], aw0 = a[3], bx1 = b[4], by1 = b[5], bz1 = b[6], bw1 = b[7], ax1 = a[4], ay1 = a[5], az1 = a[6], aw1 = a[7], bx0 = b[0], by0 = b[1], bz0 = b[2], bw0 = b[3];
      out[0] = ax0 * bw0 + aw0 * bx0 + ay0 * bz0 - az0 * by0;
      out[1] = ay0 * bw0 + aw0 * by0 + az0 * bx0 - ax0 * bz0;
      out[2] = az0 * bw0 + aw0 * bz0 + ax0 * by0 - ay0 * bx0;
      out[3] = aw0 * bw0 - ax0 * bx0 - ay0 * by0 - az0 * bz0;
      out[4] = ax0 * bw1 + aw0 * bx1 + ay0 * bz1 - az0 * by1 + ax1 * bw0 + aw1 * bx0 + ay1 * bz0 - az1 * by0;
      out[5] = ay0 * bw1 + aw0 * by1 + az0 * bx1 - ax0 * bz1 + ay1 * bw0 + aw1 * by0 + az1 * bx0 - ax1 * bz0;
      out[6] = az0 * bw1 + aw0 * bz1 + ax0 * by1 - ay0 * bx1 + az1 * bw0 + aw1 * bz0 + ax1 * by0 - ay1 * bx0;
      out[7] = aw0 * bw1 - ax0 * bx1 - ay0 * by1 - az0 * bz1 + aw1 * bw0 - ax1 * bx0 - ay1 * by0 - az1 * bz0;
      return out;
    }
    var mul = exports2.mul = multiply;
    function scale(out, a, b) {
      out[0] = a[0] * b;
      out[1] = a[1] * b;
      out[2] = a[2] * b;
      out[3] = a[3] * b;
      out[4] = a[4] * b;
      out[5] = a[5] * b;
      out[6] = a[6] * b;
      out[7] = a[7] * b;
      return out;
    }
    var dot = exports2.dot = quat.dot;
    function lerp(out, a, b, t) {
      var mt = 1 - t;
      if (dot(a, b) < 0) t = -t;
      out[0] = a[0] * mt + b[0] * t;
      out[1] = a[1] * mt + b[1] * t;
      out[2] = a[2] * mt + b[2] * t;
      out[3] = a[3] * mt + b[3] * t;
      out[4] = a[4] * mt + b[4] * t;
      out[5] = a[5] * mt + b[5] * t;
      out[6] = a[6] * mt + b[6] * t;
      out[7] = a[7] * mt + b[7] * t;
      return out;
    }
    function invert(out, a) {
      var sqlen = squaredLength(a);
      out[0] = -a[0] / sqlen;
      out[1] = -a[1] / sqlen;
      out[2] = -a[2] / sqlen;
      out[3] = a[3] / sqlen;
      out[4] = -a[4] / sqlen;
      out[5] = -a[5] / sqlen;
      out[6] = -a[6] / sqlen;
      out[7] = a[7] / sqlen;
      return out;
    }
    function conjugate(out, a) {
      out[0] = -a[0];
      out[1] = -a[1];
      out[2] = -a[2];
      out[3] = a[3];
      out[4] = -a[4];
      out[5] = -a[5];
      out[6] = -a[6];
      out[7] = a[7];
      return out;
    }
    var length = exports2.length = quat.length;
    var len = exports2.len = length;
    var squaredLength = exports2.squaredLength = quat.squaredLength;
    var sqrLen = exports2.sqrLen = squaredLength;
    function normalize(out, a) {
      var magnitude = squaredLength(a);
      if (magnitude > 0) {
        magnitude = Math.sqrt(magnitude);
        var a0 = a[0] / magnitude;
        var a1 = a[1] / magnitude;
        var a2 = a[2] / magnitude;
        var a3 = a[3] / magnitude;
        var b0 = a[4];
        var b1 = a[5];
        var b2 = a[6];
        var b3 = a[7];
        var a_dot_b = a0 * b0 + a1 * b1 + a2 * b2 + a3 * b3;
        out[0] = a0;
        out[1] = a1;
        out[2] = a2;
        out[3] = a3;
        out[4] = (b0 - a0 * a_dot_b) / magnitude;
        out[5] = (b1 - a1 * a_dot_b) / magnitude;
        out[6] = (b2 - a2 * a_dot_b) / magnitude;
        out[7] = (b3 - a3 * a_dot_b) / magnitude;
      }
      return out;
    }
    function str(a) {
      return "quat2(" + a[0] + ", " + a[1] + ", " + a[2] + ", " + a[3] + ", " + a[4] + ", " + a[5] + ", " + a[6] + ", " + a[7] + ")";
    }
    function exactEquals(a, b) {
      return a[0] === b[0] && a[1] === b[1] && a[2] === b[2] && a[3] === b[3] && a[4] === b[4] && a[5] === b[5] && a[6] === b[6] && a[7] === b[7];
    }
    function equals(a, b) {
      var a0 = a[0], a1 = a[1], a2 = a[2], a3 = a[3], a4 = a[4], a5 = a[5], a6 = a[6], a7 = a[7];
      var b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3], b4 = b[4], b5 = b[5], b6 = b[6], b7 = b[7];
      return Math.abs(a0 - b0) <= glMatrix.EPSILON * Math.max(1, Math.abs(a0), Math.abs(b0)) && Math.abs(a1 - b1) <= glMatrix.EPSILON * Math.max(1, Math.abs(a1), Math.abs(b1)) && Math.abs(a2 - b2) <= glMatrix.EPSILON * Math.max(1, Math.abs(a2), Math.abs(b2)) && Math.abs(a3 - b3) <= glMatrix.EPSILON * Math.max(1, Math.abs(a3), Math.abs(b3)) && Math.abs(a4 - b4) <= glMatrix.EPSILON * Math.max(1, Math.abs(a4), Math.abs(b4)) && Math.abs(a5 - b5) <= glMatrix.EPSILON * Math.max(1, Math.abs(a5), Math.abs(b5)) && Math.abs(a6 - b6) <= glMatrix.EPSILON * Math.max(1, Math.abs(a6), Math.abs(b6)) && Math.abs(a7 - b7) <= glMatrix.EPSILON * Math.max(1, Math.abs(a7), Math.abs(b7));
    }
  }
});

// {TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/vec2.js
var require_vec2 = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/vec2.js"(exports2) {
    "use strict";
    function _typeof(o) {
      "@babel/helpers - typeof";
      return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function(o2) {
        return typeof o2;
      } : function(o2) {
        return o2 && "function" == typeof Symbol && o2.constructor === Symbol && o2 !== Symbol.prototype ? "symbol" : typeof o2;
      }, _typeof(o);
    }
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.add = add;
    exports2.angle = angle;
    exports2.ceil = ceil;
    exports2.clone = clone;
    exports2.copy = copy;
    exports2.create = create;
    exports2.cross = cross;
    exports2.dist = void 0;
    exports2.distance = distance;
    exports2.div = void 0;
    exports2.divide = divide;
    exports2.dot = dot;
    exports2.equals = equals;
    exports2.exactEquals = exactEquals;
    exports2.floor = floor;
    exports2.forEach = void 0;
    exports2.fromValues = fromValues;
    exports2.inverse = inverse;
    exports2.len = void 0;
    exports2.length = length;
    exports2.lerp = lerp;
    exports2.max = max;
    exports2.min = min;
    exports2.mul = void 0;
    exports2.multiply = multiply;
    exports2.negate = negate;
    exports2.normalize = normalize;
    exports2.random = random;
    exports2.rotate = rotate;
    exports2.round = round;
    exports2.scale = scale;
    exports2.scaleAndAdd = scaleAndAdd;
    exports2.set = set;
    exports2.signedAngle = signedAngle;
    exports2.sqrLen = exports2.sqrDist = void 0;
    exports2.squaredDistance = squaredDistance;
    exports2.squaredLength = squaredLength;
    exports2.str = str;
    exports2.sub = void 0;
    exports2.subtract = subtract;
    exports2.transformMat2 = transformMat2;
    exports2.transformMat2d = transformMat2d;
    exports2.transformMat3 = transformMat3;
    exports2.transformMat4 = transformMat4;
    exports2.zero = zero;
    var glMatrix = _interopRequireWildcard(require_common());
    function _interopRequireWildcard(e, t) {
      if ("function" == typeof WeakMap) var r = /* @__PURE__ */ new WeakMap(), n = /* @__PURE__ */ new WeakMap();
      return (_interopRequireWildcard = function _interopRequireWildcard2(e2, t2) {
        if (!t2 && e2 && e2.__esModule) return e2;
        var o, i, f = { __proto__: null, "default": e2 };
        if (null === e2 || "object" != _typeof(e2) && "function" != typeof e2) return f;
        if (o = t2 ? n : r) {
          if (o.has(e2)) return o.get(e2);
          o.set(e2, f);
        }
        for (var _t in e2) "default" !== _t && {}.hasOwnProperty.call(e2, _t) && ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e2, _t)) && (i.get || i.set) ? o(f, _t, i) : f[_t] = e2[_t]);
        return f;
      })(e, t);
    }
    function create() {
      var out = new glMatrix.ARRAY_TYPE(2);
      if (glMatrix.ARRAY_TYPE != Float32Array) {
        out[0] = 0;
        out[1] = 0;
      }
      return out;
    }
    function clone(a) {
      var out = new glMatrix.ARRAY_TYPE(2);
      out[0] = a[0];
      out[1] = a[1];
      return out;
    }
    function fromValues(x, y) {
      var out = new glMatrix.ARRAY_TYPE(2);
      out[0] = x;
      out[1] = y;
      return out;
    }
    function copy(out, a) {
      out[0] = a[0];
      out[1] = a[1];
      return out;
    }
    function set(out, x, y) {
      out[0] = x;
      out[1] = y;
      return out;
    }
    function add(out, a, b) {
      out[0] = a[0] + b[0];
      out[1] = a[1] + b[1];
      return out;
    }
    function subtract(out, a, b) {
      out[0] = a[0] - b[0];
      out[1] = a[1] - b[1];
      return out;
    }
    function multiply(out, a, b) {
      out[0] = a[0] * b[0];
      out[1] = a[1] * b[1];
      return out;
    }
    function divide(out, a, b) {
      out[0] = a[0] / b[0];
      out[1] = a[1] / b[1];
      return out;
    }
    function ceil(out, a) {
      out[0] = Math.ceil(a[0]);
      out[1] = Math.ceil(a[1]);
      return out;
    }
    function floor(out, a) {
      out[0] = Math.floor(a[0]);
      out[1] = Math.floor(a[1]);
      return out;
    }
    function min(out, a, b) {
      out[0] = Math.min(a[0], b[0]);
      out[1] = Math.min(a[1], b[1]);
      return out;
    }
    function max(out, a, b) {
      out[0] = Math.max(a[0], b[0]);
      out[1] = Math.max(a[1], b[1]);
      return out;
    }
    function round(out, a) {
      out[0] = glMatrix.round(a[0]);
      out[1] = glMatrix.round(a[1]);
      return out;
    }
    function scale(out, a, b) {
      out[0] = a[0] * b;
      out[1] = a[1] * b;
      return out;
    }
    function scaleAndAdd(out, a, b, scale2) {
      out[0] = a[0] + b[0] * scale2;
      out[1] = a[1] + b[1] * scale2;
      return out;
    }
    function distance(a, b) {
      var x = b[0] - a[0], y = b[1] - a[1];
      return Math.sqrt(x * x + y * y);
    }
    function squaredDistance(a, b) {
      var x = b[0] - a[0], y = b[1] - a[1];
      return x * x + y * y;
    }
    function length(a) {
      var x = a[0], y = a[1];
      return Math.sqrt(x * x + y * y);
    }
    function squaredLength(a) {
      var x = a[0], y = a[1];
      return x * x + y * y;
    }
    function negate(out, a) {
      out[0] = -a[0];
      out[1] = -a[1];
      return out;
    }
    function inverse(out, a) {
      out[0] = 1 / a[0];
      out[1] = 1 / a[1];
      return out;
    }
    function normalize(out, a) {
      var x = a[0], y = a[1];
      var len2 = x * x + y * y;
      if (len2 > 0) {
        len2 = 1 / Math.sqrt(len2);
      }
      out[0] = a[0] * len2;
      out[1] = a[1] * len2;
      return out;
    }
    function dot(a, b) {
      return a[0] * b[0] + a[1] * b[1];
    }
    function cross(out, a, b) {
      var z = a[0] * b[1] - a[1] * b[0];
      out[0] = out[1] = 0;
      out[2] = z;
      return out;
    }
    function lerp(out, a, b, t) {
      var ax = a[0], ay = a[1];
      out[0] = ax + t * (b[0] - ax);
      out[1] = ay + t * (b[1] - ay);
      return out;
    }
    function random(out, scale2) {
      scale2 = scale2 === void 0 ? 1 : scale2;
      var r = glMatrix.RANDOM() * 2 * Math.PI;
      out[0] = Math.cos(r) * scale2;
      out[1] = Math.sin(r) * scale2;
      return out;
    }
    function transformMat2(out, a, m) {
      var x = a[0], y = a[1];
      out[0] = m[0] * x + m[2] * y;
      out[1] = m[1] * x + m[3] * y;
      return out;
    }
    function transformMat2d(out, a, m) {
      var x = a[0], y = a[1];
      out[0] = m[0] * x + m[2] * y + m[4];
      out[1] = m[1] * x + m[3] * y + m[5];
      return out;
    }
    function transformMat3(out, a, m) {
      var x = a[0], y = a[1];
      out[0] = m[0] * x + m[3] * y + m[6];
      out[1] = m[1] * x + m[4] * y + m[7];
      return out;
    }
    function transformMat4(out, a, m) {
      var x = a[0];
      var y = a[1];
      out[0] = m[0] * x + m[4] * y + m[12];
      out[1] = m[1] * x + m[5] * y + m[13];
      return out;
    }
    function rotate(out, a, b, rad) {
      var p0 = a[0] - b[0], p1 = a[1] - b[1], sinC = Math.sin(rad), cosC = Math.cos(rad);
      out[0] = p0 * cosC - p1 * sinC + b[0];
      out[1] = p0 * sinC + p1 * cosC + b[1];
      return out;
    }
    function angle(a, b) {
      var ax = a[0], ay = a[1], bx = b[0], by = b[1];
      return Math.abs(Math.atan2(ay * bx - ax * by, ax * bx + ay * by));
    }
    function signedAngle(a, b) {
      var ax = a[0], ay = a[1], bx = b[0], by = b[1];
      return Math.atan2(ax * by - ay * bx, ax * bx + ay * by);
    }
    function zero(out) {
      out[0] = 0;
      out[1] = 0;
      return out;
    }
    function str(a) {
      return "vec2(" + a[0] + ", " + a[1] + ")";
    }
    function exactEquals(a, b) {
      return a[0] === b[0] && a[1] === b[1];
    }
    function equals(a, b) {
      var a0 = a[0], a1 = a[1];
      var b0 = b[0], b1 = b[1];
      return Math.abs(a0 - b0) <= glMatrix.EPSILON * Math.max(1, Math.abs(a0), Math.abs(b0)) && Math.abs(a1 - b1) <= glMatrix.EPSILON * Math.max(1, Math.abs(a1), Math.abs(b1));
    }
    var len = exports2.len = length;
    var sub = exports2.sub = subtract;
    var mul = exports2.mul = multiply;
    var div = exports2.div = divide;
    var dist = exports2.dist = distance;
    var sqrDist = exports2.sqrDist = squaredDistance;
    var sqrLen = exports2.sqrLen = squaredLength;
    var forEach = exports2.forEach = (function() {
      var vec = create();
      return function(a, stride, offset, count, fn, arg) {
        var i, l;
        if (!stride) {
          stride = 2;
        }
        if (!offset) {
          offset = 0;
        }
        if (count) {
          l = Math.min(count * stride + offset, a.length);
        } else {
          l = a.length;
        }
        for (i = offset; i < l; i += stride) {
          vec[0] = a[i];
          vec[1] = a[i + 1];
          fn(vec, vec, arg);
          a[i] = vec[0];
          a[i + 1] = vec[1];
        }
        return a;
      };
    })();
  }
});

// {TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/index.js
var require_cjs = __commonJS({
  "{TSPS_CHECKOUT}/client/node_modules/gl-matrix/cjs/index.js"(exports2) {
    "use strict";
    function _typeof(o) {
      "@babel/helpers - typeof";
      return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function(o2) {
        return typeof o2;
      } : function(o2) {
        return o2 && "function" == typeof Symbol && o2.constructor === Symbol && o2 !== Symbol.prototype ? "symbol" : typeof o2;
      }, _typeof(o);
    }
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.vec4 = exports2.vec3 = exports2.vec2 = exports2.quat2 = exports2.quat = exports2.mat4 = exports2.mat3 = exports2.mat2d = exports2.mat2 = exports2.glMatrix = void 0;
    var glMatrix = _interopRequireWildcard(require_common());
    exports2.glMatrix = glMatrix;
    var mat2 = _interopRequireWildcard(require_mat2());
    exports2.mat2 = mat2;
    var mat2d = _interopRequireWildcard(require_mat2d());
    exports2.mat2d = mat2d;
    var mat3 = _interopRequireWildcard(require_mat3());
    exports2.mat3 = mat3;
    var mat42 = _interopRequireWildcard(require_mat4());
    exports2.mat4 = mat42;
    var quat = _interopRequireWildcard(require_quat());
    exports2.quat = quat;
    var quat2 = _interopRequireWildcard(require_quat2());
    exports2.quat2 = quat2;
    var vec2 = _interopRequireWildcard(require_vec2());
    exports2.vec2 = vec2;
    var vec32 = _interopRequireWildcard(require_vec3());
    exports2.vec3 = vec32;
    var vec4 = _interopRequireWildcard(require_vec4());
    exports2.vec4 = vec4;
    function _interopRequireWildcard(e, t) {
      if ("function" == typeof WeakMap) var r = /* @__PURE__ */ new WeakMap(), n = /* @__PURE__ */ new WeakMap();
      return (_interopRequireWildcard = function _interopRequireWildcard2(e2, t2) {
        if (!t2 && e2 && e2.__esModule) return e2;
        var o, i, f = { __proto__: null, "default": e2 };
        if (null === e2 || "object" != _typeof(e2) && "function" != typeof e2) return f;
        if (o = t2 ? n : r) {
          if (o.has(e2)) return o.get(e2);
          o.set(e2, f);
        }
        for (var _t in e2) "default" !== _t && {}.hasOwnProperty.call(e2, _t) && ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e2, _t)) && (i.get || i.set) ? o(f, _t, i) : f[_t] = e2[_t]);
        return f;
      })(e, t);
    }
  }
});

// {TSPS_CHECKOUT}/client/rs/model/seq/SeqTransformType.ts
var init_SeqTransformType = __esm({
  "{TSPS_CHECKOUT}/client/rs/model/seq/SeqTransformType.ts"() {
    "use strict";
  }
});

// {TSPS_CHECKOUT}/client/rs/model/Model.ts
var import_gl_matrix, scaleVector, Model;
var init_Model = __esm({
  "{TSPS_CHECKOUT}/client/rs/model/Model.ts"() {
    "use strict";
    import_gl_matrix = __toESM(require_cjs());
    init_MathConstants();
    init_Entity();
    init_ModelData();
    init_SeqTransformType();
    scaleVector = import_gl_matrix.vec3.create();
    Model = class _Model extends Entity {
      constructor() {
        super();
        this.version = 0;
        this.fractionalAnimation = false;
        this.changedLight = false;
        this.contourHeight = 0;
        // Color override fields (for damage/poison/freeze tints)
        // Reference: player-animation.md lines 1144-1151
        this.overrideHue = 0;
        // 0-127
        this.overrideSaturation = 0;
        // 0-127
        this.overrideLuminance = 0;
        // 0-127
        this.overrideAmount = 0;
        this.verticesCount = 0;
        this.usedVertexCount = 0;
        this.faceCount = 0;
        this.priority = 0;
        this.texTriangleCount = 0;
        this.isClickable = false;
        this.xMidOffset = -1;
        this.yMidOffset = -1;
        this.zMidOffset = -1;
      }
      static {
        this.animateOriginX = 0;
      }
      static {
        this.animateOriginY = 0;
      }
      static {
        this.animateOriginZ = 0;
      }
      static {
        this.sketetalTransformMatrix = import_gl_matrix.mat4.create();
      }
      static {
        this.skeletalScalingMatrix = import_gl_matrix.mat4.create();
      }
      static {
        this.skeletalBoneMatrix = import_gl_matrix.mat4.create();
      }
      /** Preserve sub-unit animation transforms without changing unanimated cache geometry. */
      enableFractionalAnimation() {
        if (this.fractionalAnimation) return;
        this.fractionalAnimation = true;
        if (this.verticesX) this.verticesX = Float32Array.from(this.verticesX);
        if (this.verticesY) this.verticesY = Float32Array.from(this.verticesY);
        if (this.verticesZ) this.verticesZ = Float32Array.from(this.verticesZ);
      }
      // 0-255 (0 = no override, 255 = full override)
      static copy(model) {
        return _Model.merge([model], 1);
      }
      static copyAnimated(model, shallowTransparencies, shallowColors) {
        const copy = Object.assign(Object.create(Object.getPrototypeOf(model)), model);
        const Coordinates = model.fractionalAnimation ? Float32Array : Int32Array;
        copy.verticesX = new Coordinates(model.verticesCount);
        copy.verticesY = new Coordinates(model.verticesCount);
        copy.verticesZ = new Coordinates(model.verticesCount);
        for (let i = 0; i < model.verticesCount; i++) {
          copy.verticesX[i] = model.verticesX[i];
          copy.verticesY[i] = model.verticesY[i];
          copy.verticesZ[i] = model.verticesZ[i];
        }
        if (shallowTransparencies) {
          copy.faceAlphas = model.faceAlphas;
        } else {
          copy.faceAlphas = new Int8Array(model.faceCount);
          if (model.faceAlphas) {
            for (let i = 0; i < model.faceCount; i++) {
              copy.faceAlphas[i] = model.faceAlphas[i];
            }
          } else {
            copy.faceAlphas.fill(0);
          }
        }
        if (!shallowColors) {
          copy.faceColors = new Uint16Array(model.faceCount);
          copy.faceColors1 = new Int32Array(model.faceCount);
          copy.faceColors2 = new Int32Array(model.faceCount);
          copy.faceColors3 = new Int32Array(model.faceCount);
          for (let i = 0; i < model.faceCount; i++) {
            copy.faceColors[i] = model.faceColors[i];
            copy.faceColors1[i] = model.faceColors1[i];
            copy.faceColors2[i] = model.faceColors2[i];
            copy.faceColors3[i] = model.faceColors3[i];
          }
        }
        return copy;
      }
      static merge(models, count) {
        const model = new _Model();
        model.merge(models, count);
        return model;
      }
      static resetAnimateOrigin() {
        _Model.animateOriginX = 0;
        _Model.animateOriginY = 0;
        _Model.animateOriginZ = 0;
      }
      merge(models, count) {
        let hasRenderPriority = false;
        let hasAlpha = false;
        let hasTexture = false;
        let hasTextureCoord = false;
        this.verticesCount = 0;
        this.faceCount = 0;
        this.texTriangleCount = 0;
        this.priority = -1;
        for (let i = 0; i < count; i++) {
          const model = models[i];
          if (model) {
            this.verticesCount += model.verticesCount;
            this.faceCount += model.faceCount;
            this.texTriangleCount += model.texTriangleCount;
            if (model.faceRenderPriorities) {
              hasRenderPriority = true;
            } else {
              if (this.priority === -1) {
                this.priority = model.priority;
              }
              if (this.priority !== model.priority) {
                hasRenderPriority = true;
              }
            }
            hasAlpha ||= !!model.faceAlphas;
            hasTexture ||= !!model.faceTextures;
            hasTextureCoord ||= !!model.textureCoords;
          }
        }
        this.verticesX = new Int32Array(this.verticesCount);
        this.verticesY = new Int32Array(this.verticesCount);
        this.verticesZ = new Int32Array(this.verticesCount);
        this.indices1 = new Int32Array(this.faceCount);
        this.indices2 = new Int32Array(this.faceCount);
        this.indices3 = new Int32Array(this.faceCount);
        this.faceColors1 = new Int32Array(this.faceCount);
        this.faceColors2 = new Int32Array(this.faceCount);
        this.faceColors3 = new Int32Array(this.faceCount);
        this.faceColors = new Uint16Array(this.faceCount);
        if (hasRenderPriority) {
          this.faceRenderPriorities = new Int8Array(this.faceCount);
        }
        if (hasAlpha) {
          this.faceAlphas = new Int8Array(this.faceCount);
        }
        if (hasTexture) {
          this.faceTextures = new Int16Array(this.faceCount);
        }
        if (hasTextureCoord) {
          this.textureCoords = new Int8Array(this.faceCount);
        }
        if (this.texTriangleCount > 0) {
          this.textureMappingP = new Int32Array(this.texTriangleCount);
          this.textureMappingM = new Int32Array(this.texTriangleCount);
          this.textureMappingN = new Int32Array(this.texTriangleCount);
          this.textureScaleX = new Int32Array(this.texTriangleCount);
          this.textureScaleY = new Int32Array(this.texTriangleCount);
          this.textureScaleZ = new Int32Array(this.texTriangleCount);
          this.textureRotation = new Int8Array(this.texTriangleCount);
          this.textureDirection = new Int8Array(this.texTriangleCount);
          this.textureSpeed = new Int32Array(this.texTriangleCount);
          this.textureTransU = new Int32Array(this.texTriangleCount);
          this.textureTransV = new Int32Array(this.texTriangleCount);
        }
        this.verticesCount = 0;
        this.faceCount = 0;
        this.texTriangleCount = 0;
        for (let i = 0; i < count; i++) {
          const model = models[i];
          if (model) {
            for (let f = 0; f < model.faceCount; f++) {
              this.indices1[this.faceCount] = this.verticesCount + model.indices1[f];
              this.indices2[this.faceCount] = this.verticesCount + model.indices2[f];
              this.indices3[this.faceCount] = this.verticesCount + model.indices3[f];
              this.faceColors1[this.faceCount] = model.faceColors1[f];
              this.faceColors2[this.faceCount] = model.faceColors2[f];
              this.faceColors3[this.faceCount] = model.faceColors3[f];
              this.faceColors[this.faceCount] = model.faceColors[f];
              if (hasRenderPriority) {
                if (model.faceRenderPriorities) {
                  this.faceRenderPriorities[this.faceCount] = model.faceRenderPriorities[f];
                } else {
                  this.faceRenderPriorities[this.faceCount] = model.priority;
                }
              }
              if (hasAlpha && model.faceAlphas) {
                this.faceAlphas[this.faceCount] = model.faceAlphas[f];
              }
              if (hasTexture && this.faceTextures) {
                if (model.faceTextures) {
                  this.faceTextures[this.faceCount] = model.faceTextures[f];
                } else {
                  this.faceTextures[this.faceCount] = -1;
                }
              }
              if (hasTextureCoord) {
                if (model.textureCoords && model.textureCoords[f] !== -1) {
                  this.textureCoords[this.faceCount] = this.texTriangleCount + model.textureCoords[f];
                } else {
                  this.textureCoords[this.faceCount] = -1;
                }
              }
              this.faceCount++;
            }
            for (let v = 0; v < model.texTriangleCount; v++) {
              this.textureMappingP[this.texTriangleCount] = this.verticesCount + model.textureMappingP[v];
              this.textureMappingM[this.texTriangleCount] = this.verticesCount + model.textureMappingM[v];
              this.textureMappingN[this.texTriangleCount] = this.verticesCount + model.textureMappingN[v];
              this.texTriangleCount++;
            }
            for (let v = 0; v < model.verticesCount; v++) {
              this.verticesX[this.verticesCount] = model.verticesX[v];
              this.verticesY[this.verticesCount] = model.verticesY[v];
              this.verticesZ[this.verticesCount] = model.verticesZ[v];
              this.verticesCount++;
            }
          }
        }
        this.usedVertexCount = this.verticesCount;
      }
      calculateBoundsCylinder() {
        if (this.boundsType !== 1) {
          this.boundsType = 1;
          this.height = 0;
          this.bottomY = 0;
          this.xzRadius = 0;
          for (let i = 0; i < this.usedVertexCount; i++) {
            const vertX = this.verticesX[i];
            const vertY = this.verticesY[i];
            const vertZ = this.verticesZ[i];
            if (-vertY > this.height) {
              this.height = -vertY;
            }
            if (this.contourVerticesY) {
              const contourY = this.contourVerticesY[i];
              if (-contourY > this.contourHeight) {
                this.contourHeight = -contourY;
              }
            }
            if (vertY > this.bottomY) {
              this.bottomY = vertY;
            }
            const var5 = vertX * vertX + vertZ * vertZ;
            if (var5 > this.xzRadius) {
              this.xzRadius = var5;
            }
          }
          if (!this.contourVerticesY) {
            this.contourHeight = this.height;
          }
          this.xzRadius = Math.sqrt(this.xzRadius) + 0.99 | 0;
          this.radius = Math.sqrt(this.xzRadius * this.xzRadius + this.height * this.height) + 0.99 | 0;
          this.diameter = this.radius + (Math.sqrt(this.xzRadius * this.xzRadius + this.bottomY * this.bottomY) + 0.99) | 0;
        }
      }
      calculateBounds() {
        if (this.boundsType !== 2) {
          this.height = 0;
          this.minHeight = 0;
          this.minX = 999999;
          this.maxX = -999999;
          this.minY = 999999;
          this.maxY = -999999;
          this.minZ = 99999;
          this.maxZ = -99999;
          const verticesY = this.contourVerticesY ?? this.verticesY;
          for (let i = 0; i < this.usedVertexCount; i++) {
            const vertX = this.verticesX[i];
            const vertY = verticesY[i];
            const vertZ = this.verticesZ[i];
            if (vertX < this.minX) {
              this.minX = vertX;
            }
            if (vertX > this.maxX) {
              this.maxX = vertX;
            }
            if (this.minY > vertY) {
              this.minY = vertY;
            }
            if (this.maxY < vertY) {
              this.maxY = vertY;
            }
            if (vertZ < this.minZ) {
              this.minZ = vertZ;
            }
            if (vertZ > this.maxZ) {
              this.maxZ = vertZ;
            }
            if (-vertY > this.height) {
              this.height = -vertY;
            }
            if (vertY > this.minHeight) {
              this.minHeight = vertY;
            }
          }
          this.boundsType = 2;
        }
      }
      invalidateBounds() {
        this.boundsType = 0;
        this.xMidOffset = -1;
      }
      contourGround(type, param, heightMap, heightMapAbove, sceneX, sceneHeight, sceneZ, createNew = true) {
        if (this.usedVertexCount === 0) {
          return this;
        }
        this.calculateBounds();
        let startX = sceneX + this.minX;
        let endX = sceneX + this.maxX;
        let startY = sceneZ + this.minZ;
        let endY = sceneZ + this.maxZ;
        if ((type === 1 || type === 2 || type === 3 || type === 5) && (startX < 0 || endX + 128 >> 7 >= heightMap.length || startY < 0 || endY + 128 >> 7 >= heightMap[0].length)) {
          return this;
        }
        if (type === 4 || type === 5) {
          if (heightMapAbove === void 0) {
            return this;
          }
          if (startX < 0 || endX + 128 >> 7 >= heightMapAbove.length || startY < 0 || endY + 128 >> 7 >= heightMapAbove[0].length) {
            return this;
          }
        } else {
          startX >>= 7;
          endX = endX + 127 >> 7;
          startY >>= 7;
          endY = endY + 127 >> 7;
          if (heightMap[startX][startY] === sceneHeight && heightMap[endX][startY] === sceneHeight && heightMap[startX][endY] === sceneHeight && heightMap[endX][endY] === sceneHeight) {
            return this;
          }
        }
        let model = this;
        if (createNew) {
          model = new _Model();
          model.verticesCount = this.verticesCount;
          model.usedVertexCount = this.usedVertexCount;
          model.faceCount = this.faceCount;
          model.texTriangleCount = this.texTriangleCount;
          model.verticesX = this.verticesX;
          model.verticesZ = this.verticesZ;
          model.indices1 = this.indices1;
          model.indices2 = this.indices2;
          model.indices3 = this.indices3;
          model.faceColors1 = this.faceColors1;
          model.faceColors2 = this.faceColors2;
          model.faceColors3 = this.faceColors3;
          model.faceColors = this.faceColors;
          model.faceRenderPriorities = this.faceRenderPriorities;
          model.faceRenderLayers = this.faceRenderLayers;
          model.faceAlphas = this.faceAlphas;
          model.textureCoords = this.textureCoords;
          model.faceTextures = this.faceTextures;
          model.priority = this.priority;
          model.textureMappingP = this.textureMappingP;
          model.textureMappingM = this.textureMappingM;
          model.textureMappingN = this.textureMappingN;
          model.textureScaleX = this.textureScaleX;
          model.textureScaleY = this.textureScaleY;
          model.textureScaleZ = this.textureScaleZ;
          model.textureRotation = this.textureRotation;
          model.textureDirection = this.textureDirection;
          model.textureSpeed = this.textureSpeed;
          model.textureTransU = this.textureTransU;
          model.textureTransV = this.textureTransV;
          model.uvs = this.uvs;
          model.vertexLabels = this.vertexLabels;
          model.faceLabels = this.faceLabels;
          model.isClickable = this.isClickable;
          model.verticesY = this.verticesY;
        }
        model.contourVerticesY = new Int32Array(model.verticesCount);
        if (type === 1) {
          for (let i = 0; i < model.usedVertexCount; i++) {
            const vx = this.verticesX[i] + sceneX;
            const vz = this.verticesZ[i] + sceneZ;
            const rx = vx & 127;
            const rz = vz & 127;
            const tx = vx >> 7;
            const tz = vz >> 7;
            const h0 = heightMap[tx][tz] * (128 - rx) + heightMap[tx + 1][tz] * rx >> 7;
            const h1 = heightMap[tx][tz + 1] * (128 - rx) + heightMap[tx + 1][tz + 1] * rx >> 7;
            const height = h0 * (128 - rz) + h1 * rz >> 7;
            model.contourVerticesY[i] = this.verticesY[i] + height - sceneHeight;
          }
          for (let i = model.usedVertexCount; i < model.verticesCount; i++) {
            const vx = this.verticesX[i] + sceneX;
            const vz = this.verticesZ[i] + sceneZ;
            const rx = vx & 127;
            const rz = vz & 127;
            const tx = vx >> 7;
            const tz = vz >> 7;
            if (tx >= 0 && tx < heightMap.length - 1 && tz >= 0 && tz < heightMap[0].length - 1) {
              const h0 = heightMap[tx][tz] * (128 - rx) + heightMap[tx + 1][tz] * rx >> 7;
              const h1 = heightMap[tx][tz + 1] * (128 - rx) + heightMap[tx + 1][tz + 1] * rx >> 7;
              const height = h0 * (128 - rz) + h1 * rz >> 7;
              model.contourVerticesY[i] = this.verticesY[i] + height - sceneHeight;
            } else {
              model.contourVerticesY[i] = this.verticesY[i];
            }
          }
        } else if (type === 2) {
          for (let i = 0; i < model.usedVertexCount; i++) {
            const yRatio = (this.verticesY[i] << 16) / this.minY | 0;
            if (yRatio < param) {
              const vx = this.verticesX[i] + sceneX;
              const vz = this.verticesZ[i] + sceneZ;
              const rx = vx & 127;
              const rz = vz & 127;
              const tx = vx >> 7;
              const tz = vz >> 7;
              const h0 = heightMap[tx][tz] * (128 - rx) + heightMap[tx + 1][tz] * rx >> 7;
              const h1 = heightMap[tx][tz + 1] * (128 - rx) + heightMap[tx + 1][tz + 1] * rx >> 7;
              const height = h0 * (128 - rz) + h1 * rz >> 7;
              model.contourVerticesY[i] = this.verticesY[i] + (height - sceneHeight) * (param - yRatio) / param;
            } else {
              model.contourVerticesY[i] = this.verticesY[i];
            }
          }
          for (let i = model.usedVertexCount; i < model.verticesCount; i++) {
            const yRatio = (this.verticesY[i] << 16) / this.minY | 0;
            if (yRatio < param) {
              const vx = this.verticesX[i] + sceneX;
              const vz = this.verticesZ[i] + sceneZ;
              const rx = vx & 127;
              const rz = vz & 127;
              const tx = vx >> 7;
              const tz = vz >> 7;
              if (tx >= 0 && tx < heightMap.length - 1 && tz >= 0 && tz < heightMap[0].length - 1) {
                const h0 = heightMap[tx][tz] * (128 - rx) + heightMap[tx + 1][tz] * rx >> 7;
                const h1 = heightMap[tx][tz + 1] * (128 - rx) + heightMap[tx + 1][tz + 1] * rx >> 7;
                const height = h0 * (128 - rz) + h1 * rz >> 7;
                model.contourVerticesY[i] = this.verticesY[i] + (height - sceneHeight) * (param - yRatio) / param;
              }
            } else {
              model.contourVerticesY[i] = this.verticesY[i];
            }
          }
        } else if (type === 3) {
          const clampLimit = param !== 0 ? Math.abs(param) : void 0;
          for (let i = 0; i < model.usedVertexCount; i++) {
            const vx = this.verticesX[i] + sceneX;
            const vz = this.verticesZ[i] + sceneZ;
            const rx = vx & 127;
            const rz = vz & 127;
            const tx = vx >> 7;
            const tz = vz >> 7;
            if (tx >= 0 && tx < heightMap.length - 1 && tz >= 0 && tz < heightMap[0].length - 1) {
              const h0 = heightMap[tx][tz] * (128 - rx) + heightMap[tx + 1][tz] * rx >> 7;
              const h1 = heightMap[tx][tz + 1] * (128 - rx) + heightMap[tx + 1][tz + 1] * rx >> 7;
              let delta = (h0 * (128 - rz) + h1 * rz >> 7) - sceneHeight;
              if (clampLimit !== void 0 && clampLimit > 0) {
                if (delta > clampLimit) {
                  delta = clampLimit;
                } else if (delta < -clampLimit) {
                  delta = -clampLimit;
                }
              }
              model.contourVerticesY[i] = this.verticesY[i] + delta;
            } else {
              model.contourVerticesY[i] = this.verticesY[i];
            }
          }
        } else if (type === 4) {
          const deltaY = this.maxY - this.minY;
          for (let i = 0; i < model.usedVertexCount; i++) {
            const vx = this.verticesX[i] + sceneX;
            const vz = this.verticesZ[i] + sceneZ;
            const rx = vx & 127;
            const rz = vz & 127;
            const tx = vx >> 7;
            const tz = vz >> 7;
            const h0 = heightMapAbove[tx][tz] * (128 - rx) + heightMapAbove[tx + 1][tz] * rx >> 7;
            const h1 = heightMapAbove[tx][tz + 1] * (128 - rx) + heightMapAbove[tx + 1][tz + 1] * rx >> 7;
            const height = h0 * (128 - rz) + h1 * rz >> 7;
            model.contourVerticesY[i] = this.verticesY[i] + height - sceneHeight + deltaY;
          }
        } else if (type === 5) {
          const deltaY = this.maxY - this.minY;
          for (let i = 0; i < model.usedVertexCount; i++) {
            const vx = this.verticesX[i] + sceneX;
            const vz = this.verticesZ[i] + sceneZ;
            const rx = vx & 127;
            const rz = vz & 127;
            const tx = vx >> 7;
            const tz = vz >> 7;
            let h0 = heightMap[tx][tz] * (128 - rx) + heightMap[tx + 1][tz] * rx >> 7;
            let h1 = heightMap[tx][tz + 1] * (128 - rx) + heightMap[tx + 1][tz + 1] * rx >> 7;
            const height = h0 * (128 - rz) + h1 * rz >> 7;
            h0 = heightMapAbove[tx][tz] * (128 - rx) + heightMapAbove[tx + 1][tz] * rx >> 7;
            h1 = heightMapAbove[tx][tz + 1] * (128 - rx) + heightMapAbove[tx + 1][tz + 1] * rx >> 7;
            const heightAbove = h0 * (128 - rz) + h1 * rz >> 7;
            const deltaHeight = height - heightAbove;
            model.contourVerticesY[i] = (((this.verticesY[i] << 8) / deltaY | 0) * deltaHeight >> 8) - (sceneHeight - height);
          }
        }
        model.invalidateBounds();
        return model;
      }
      rotate90() {
        for (let i = 0; i < this.verticesCount; i++) {
          const temp = this.verticesX[i];
          this.verticesX[i] = this.verticesZ[i];
          this.verticesZ[i] = -temp;
        }
        this.invalidateBounds();
      }
      rotate180() {
        for (let i = 0; i < this.verticesCount; i++) {
          this.verticesX[i] = -this.verticesX[i];
          this.verticesZ[i] = -this.verticesZ[i];
        }
        this.invalidateBounds();
      }
      rotate270() {
        for (let i = 0; i < this.verticesCount; i++) {
          const temp = this.verticesZ[i];
          this.verticesZ[i] = this.verticesX[i];
          this.verticesX[i] = -temp;
        }
        this.invalidateBounds();
      }
      rotate(angle) {
        const sin = SINE[angle];
        const cos = COSINE[angle];
        for (let i = 0; i < this.verticesCount; i++) {
          const x = sin * this.verticesZ[i] + cos * this.verticesX[i];
          const z = cos * this.verticesZ[i] - sin * this.verticesX[i];
          this.verticesX[i] = this.fractionalAnimation ? x / 65536 : x >> 16;
          this.verticesZ[i] = this.fractionalAnimation ? z / 65536 : z >> 16;
        }
        this.invalidateBounds();
      }
      translate(x, y, z) {
        for (let i = 0; i < this.verticesCount; i++) {
          this.verticesX[i] += x;
          this.verticesY[i] += y;
          this.verticesZ[i] += z;
        }
        this.invalidateBounds();
      }
      scale(x, y, z) {
        for (let i = 0; i < this.verticesCount; i++) {
          this.verticesX[i] = this.fractionalAnimation ? this.verticesX[i] * x / 128 : this.verticesX[i] * x / 128 | 0;
          this.verticesY[i] = this.fractionalAnimation ? this.verticesY[i] * y / 128 : this.verticesY[i] * y / 128 | 0;
          this.verticesZ[i] = this.fractionalAnimation ? this.verticesZ[i] * z / 128 : this.verticesZ[i] * z / 128 | 0;
        }
        this.invalidateBounds();
      }
      getXZRadius() {
        this.calculateBoundsCylinder();
        return this.xzRadius;
      }
      animateOld(frame) {
        if (this.vertexLabels && frame) {
          _Model.resetAnimateOrigin();
          const base = frame.base;
          for (let i = 0; i < frame.transformCount; i++) {
            const group = frame.transformGroups[i];
            this.transform(
              base.types[group],
              base.labels[group],
              frame.transformX[i],
              frame.transformY[i],
              frame.transformZ[i]
            );
          }
          this.postAnimate();
        }
      }
      animate(frame, nextFrame, op14) {
        if (this.vertexLabels) {
          _Model.resetAnimateOrigin();
          const base = frame.base;
          if (base !== nextFrame?.base) {
            nextFrame = void 0;
          }
          this.transformInterpolated(base, frame, nextFrame, void 0, true, op14, 65535, 0);
          this.postAnimate();
        }
      }
      animateInterpolated(frame, nextFrame, alpha, op14) {
        this.enableFractionalAnimation();
        if (this.vertexLabels) {
          _Model.resetAnimateOrigin();
          const base = frame.base;
          if (base !== nextFrame?.base) {
            nextFrame = void 0;
          }
          this.transformInterpolated(
            base,
            frame,
            nextFrame,
            void 0,
            true,
            op14,
            65535,
            alpha
          );
          this.postAnimate();
        }
      }
      applyInterleaveFrame(frame, interleave, match, op14) {
        if (!this.vertexLabels) return;
        _Model.resetAnimateOrigin();
        const base = frame.base;
        let maskIndex = 0;
        let maskValue = (interleave[maskIndex] ?? 9999999) | 0;
        maskIndex++;
        for (let i = 0; i < frame.transformCount; i++) {
          const group = frame.transformGroups[i] | 0;
          while (group > maskValue) {
            maskValue = (interleave[maskIndex] ?? 9999999) | 0;
            maskIndex++;
          }
          const type = base.types[group] | 0;
          const selected = match ? group === maskValue || type === 0 /* ORIGIN */ : group !== maskValue || type === 0 /* ORIGIN */;
          if (!selected) continue;
          const resetOriginGroup = frame.resetOriginGroups[i];
          if (resetOriginGroup !== -1) {
            this.transform(
              0 /* ORIGIN */,
              base.labels[resetOriginGroup],
              0,
              0,
              0,
              op14,
              65535
            );
          }
          this.transform(
            base.types[group],
            base.labels[group],
            frame.transformX[i],
            frame.transformY[i],
            frame.transformZ[i],
            op14,
            65535
          );
        }
      }
      applyInterleaveFrameInterpolated(frame, nextFrame, alpha, interleave, match, op14) {
        if (!this.vertexLabels) return;
        _Model.resetAnimateOrigin();
        const selected = new Array(frame.base.count).fill(false);
        for (const group of interleave) if (group < selected.length) selected[group] = true;
        this.transformInterpolated(
          frame.base,
          frame,
          nextFrame?.base === frame.base ? nextFrame : void 0,
          selected,
          match,
          op14,
          65535,
          alpha
        );
      }
      /**
       * Apply a single frame-based sequence using an interleave array.
       * Used by animation system for interleaved transforms
       * when combining cached and non-cached sequences.
       */
      animateInterleavedFrame(frame, op14, interleave, match, nextFrame, alpha) {
        if (alpha !== void 0) {
          this.enableFractionalAnimation();
          if (!interleave?.length) {
            this.animateInterpolated(frame, nextFrame, alpha, op14);
            return;
          }
          this.applyInterleaveFrameInterpolated(frame, nextFrame, alpha, interleave, match, op14);
          this.postAnimate();
          return;
        }
        if (!this.vertexLabels) return;
        if (!interleave || interleave.length === 0) {
          this.animate(frame, void 0, op14);
          return;
        }
        this.applyInterleaveFrame(frame, interleave, match, op14);
        this.postAnimate();
      }
      /**
       * Apply two frame-based sequences using the action sequence's interleave array.
       */
      animateInterleavedFrames(baseFrame, baseOp14, overlayFrame, overlayOp14, interleave) {
        if (!this.vertexLabels) return;
        if (!interleave || interleave.length === 0) {
          this.animate(baseFrame, void 0, baseOp14);
          return;
        }
        this.applyInterleaveFrame(baseFrame, interleave, false, baseOp14);
        this.applyInterleaveFrame(overlayFrame, interleave, true, overlayOp14);
        this.postAnimate();
      }
      animateInterleavedFramesInterpolated(baseFrame, nextBaseFrame, baseAlpha, baseOp14, overlayFrame, nextOverlayFrame, overlayAlpha, overlayOp14, interleave) {
        this.enableFractionalAnimation();
        if (!this.vertexLabels) return;
        if (!interleave || interleave.length === 0) {
          this.animateInterpolated(baseFrame, nextBaseFrame, baseAlpha, baseOp14);
          return;
        }
        this.applyInterleaveFrameInterpolated(
          baseFrame,
          nextBaseFrame,
          baseAlpha,
          interleave,
          false,
          baseOp14
        );
        this.applyInterleaveFrameInterpolated(
          overlayFrame,
          nextOverlayFrame,
          overlayAlpha,
          interleave,
          true,
          overlayOp14
        );
        this.postAnimate();
      }
      /**
       * Applies `frame`, or with a `nextFrame` and `alpha` > 0 a blend of the two (animation
       * smoothing): every skeleton group either frame transforms is moved `alpha` of the way from
       * its value in `frame` to its value in `nextFrame` (a group a frame leaves out counts as
       * neutral: 0, or 128 for scales). Rotations take the short way round, and a half turn snaps;
       * alpha transforms are not blended.
       */
      transformInterpolated(base, frame, nextFrame, animateLabels, condition, op14, mask, alpha) {
        if (nextFrame && alpha >= 1) {
          frame = nextFrame;
          nextFrame = void 0;
        }
        if (!nextFrame || !(alpha > 0)) {
          for (let i2 = 0; i2 < frame.transformCount; i2++) {
            const group = frame.transformGroups[i2];
            if (!this.isAnimatedGroup(base, group, animateLabels, condition)) continue;
            this.resetOrigin(base, frame.resetOriginGroups[i2], op14, mask);
            this.transform(
              base.types[group],
              base.labels[group],
              frame.transformX[i2],
              frame.transformY[i2],
              frame.transformZ[i2],
              op14,
              base.masks[group] & mask
            );
          }
          return;
        }
        const from = _Model.blendEntries(frame);
        const to = _Model.blendEntries(nextFrame);
        let i = 0;
        let n = 0;
        while (i < from.length || n < to.length) {
          const group = Math.min(from[i]?.group ?? Infinity, to[n]?.group ?? Infinity);
          const current = from[i]?.group === group ? from[i++] : void 0;
          const next = to[n]?.group === group ? to[n++] : void 0;
          if (!this.isAnimatedGroup(base, group, animateLabels, condition)) continue;
          const type = base.types[group];
          const neutral = type === 3 /* SCALE */ || type === 10 /* TYPE_10 */ ? 128 : 0;
          const x0 = current?.x ?? neutral;
          const y0 = current?.y ?? neutral;
          const z0 = current?.z ?? neutral;
          const x1 = next?.x ?? neutral;
          const y1 = next?.y ?? neutral;
          const z1 = next?.z ?? neutral;
          let x;
          let y;
          let z;
          if (type === 2 /* ROTATE */) {
            const dx = _Model.angleDelta(x0, x1);
            const dy = _Model.angleDelta(y0, y1);
            const dz = _Model.angleDelta(z0, z1);
            const flip = dx === -128 || dy === -128 || dz === -128;
            x = flip ? x0 : ((x0 + dx * alpha) % 256 + 256) % 256;
            y = flip ? y0 : ((y0 + dy * alpha) % 256 + 256) % 256;
            z = flip ? z0 : ((z0 + dz * alpha) % 256 + 256) % 256;
          } else if (type === 5 /* ALPHA */) {
            x = x0;
            y = 0;
            z = 0;
          } else {
            x = x0 + (x1 - x0) * alpha;
            y = y0 + (y1 - y0) * alpha;
            z = z0 + (z1 - z0) * alpha;
          }
          this.transform(type, base.labels[group], x, y, z, op14, base.masks[group] & mask);
        }
      }
      static {
        this.blendEntryCache = /* @__PURE__ */ new WeakMap();
      }
      /** A frame's transforms with its origin resets as origin transforms, ordered by group. */
      static blendEntries(frame) {
        let entries = _Model.blendEntryCache.get(frame);
        if (entries) return entries;
        entries = [];
        for (let i = 0; i < frame.transformCount; i++) {
          const reset = frame.resetOriginGroups[i];
          if (reset !== -1 && reset !== void 0) {
            entries.push({ group: reset, x: 0, y: 0, z: 0 });
          }
          entries.push({
            group: frame.transformGroups[i],
            x: frame.transformX[i],
            y: frame.transformY[i],
            z: frame.transformZ[i]
          });
        }
        entries.sort((a, b) => a.group - b.group);
        _Model.blendEntryCache.set(frame, entries);
        return entries;
      }
      /** The signed shortest step from one 0-255 angle to another. */
      static angleDelta(from, to) {
        const delta = to - from & 255;
        return delta >= 128 ? delta - 256 : delta;
      }
      isAnimatedGroup(base, group, animateLabels, condition) {
        return !animateLabels || animateLabels[group] === condition || base.types[group] === 0 /* ORIGIN */;
      }
      resetOrigin(base, resetOriginGroup, op14, mask) {
        if (resetOriginGroup === -1 || resetOriginGroup === void 0) return;
        this.transform(
          0 /* ORIGIN */,
          base.labels[resetOriginGroup],
          0,
          0,
          0,
          op14,
          base.masks[resetOriginGroup] & mask
        );
      }
      transform(type, labels, tx, ty, tz, op14 = false, mask = 65535) {
        if (mask !== 65535) {
        } else {
          this.transform0(type, labels, tx, ty, tz, op14);
        }
      }
      transformFractional(type, labels, tx, ty, tz) {
        if (type === 0 /* ORIGIN */) {
          _Model.resetAnimateOrigin();
          let count = 0;
          for (const label of labels) for (const v of this.vertexLabels[label] ?? []) {
            _Model.animateOriginX += this.verticesX[v];
            _Model.animateOriginY += this.verticesY[v];
            _Model.animateOriginZ += this.verticesZ[v];
            count++;
          }
          _Model.animateOriginX = tx + (count ? _Model.animateOriginX / count : 0);
          _Model.animateOriginY = ty + (count ? _Model.animateOriginY / count : 0);
          _Model.animateOriginZ = tz + (count ? _Model.animateOriginZ / count : 0);
          return;
        }
        const radians = Math.PI * 2 / 256;
        const rotate = type === 2 /* ROTATE */;
        const sx = rotate ? Math.sin(tx * radians) : 0, cx = rotate ? Math.cos(tx * radians) : 1;
        const sy = rotate ? Math.sin(ty * radians) : 0, cy = rotate ? Math.cos(ty * radians) : 1;
        const sz = rotate ? Math.sin(tz * radians) : 0, cz = rotate ? Math.cos(tz * radians) : 1;
        for (const label of labels) for (const v of this.vertexLabels[label] ?? []) {
          if (type === 1 /* TRANSLATE */) {
            this.verticesX[v] += tx;
            this.verticesY[v] += ty;
            this.verticesZ[v] += tz;
            continue;
          }
          let x = this.verticesX[v] - _Model.animateOriginX;
          let y = this.verticesY[v] - _Model.animateOriginY;
          let z = this.verticesZ[v] - _Model.animateOriginZ;
          if (type === 2 /* ROTATE */) {
            let t = sz * y + cz * x;
            y = cz * y - sz * x;
            x = t;
            t = cx * y - sx * z;
            z = sx * y + cx * z;
            y = t;
            t = sy * z + cy * x;
            z = cy * z - sy * x;
            x = t;
          } else if (type === 3 /* SCALE */) {
            x *= tx / 128;
            y *= ty / 128;
            z *= tz / 128;
          }
          this.verticesX[v] = x + _Model.animateOriginX;
          this.verticesY[v] = y + _Model.animateOriginY;
          this.verticesZ[v] = z + _Model.animateOriginZ;
        }
      }
      transform0(type, labels, tx, ty, tz, op14) {
        if (this.fractionalAnimation && type <= 3 /* SCALE */) {
          this.transformFractional(type, labels, tx, ty, tz);
          return;
        }
        switch (type) {
          case 0 /* ORIGIN */:
            _Model.resetAnimateOrigin();
            let groupVertexCount = 0;
            for (const label of labels) {
              if (label < this.vertexLabels.length) {
                for (const v of this.vertexLabels[label]) {
                  _Model.animateOriginX += this.verticesX[v];
                  _Model.animateOriginY += this.verticesY[v];
                  _Model.animateOriginZ += this.verticesZ[v];
                  groupVertexCount++;
                }
              }
            }
            if (groupVertexCount > 0) {
              _Model.animateOriginX = tx + (_Model.animateOriginX / groupVertexCount | 0);
              _Model.animateOriginY = ty + (_Model.animateOriginY / groupVertexCount | 0);
              _Model.animateOriginZ = tz + (_Model.animateOriginZ / groupVertexCount | 0);
            } else {
              _Model.animateOriginX = tx;
              _Model.animateOriginY = ty;
              _Model.animateOriginZ = tz;
            }
            break;
          case 1 /* TRANSLATE */:
            for (const label of labels) {
              if (label < this.vertexLabels.length) {
                for (const v of this.vertexLabels[label]) {
                  this.verticesX[v] += tx;
                  this.verticesY[v] += ty;
                  this.verticesZ[v] += tz;
                }
              }
            }
            break;
          case 2 /* ROTATE */:
            for (const label of labels) {
              if (label < this.vertexLabels.length) {
                for (const v of this.vertexLabels[label]) {
                  this.verticesX[v] -= _Model.animateOriginX;
                  this.verticesY[v] -= _Model.animateOriginY;
                  this.verticesZ[v] -= _Model.animateOriginZ;
                  const angleX = (tx & 255) * 8;
                  const angleY = (ty & 255) * 8;
                  const angleZ = (tz & 255) * 8;
                  if (angleZ !== 0) {
                    const sin = SINE[angleZ];
                    const cos = COSINE[angleZ];
                    const temp = sin * this.verticesY[v] + cos * this.verticesX[v] >> 16;
                    this.verticesY[v] = cos * this.verticesY[v] - sin * this.verticesX[v] >> 16;
                    this.verticesX[v] = temp;
                  }
                  if (angleX !== 0) {
                    const sin = SINE[angleX];
                    const cos = COSINE[angleX];
                    const temp = cos * this.verticesY[v] - sin * this.verticesZ[v] >> 16;
                    this.verticesZ[v] = sin * this.verticesY[v] + cos * this.verticesZ[v] >> 16;
                    this.verticesY[v] = temp;
                  }
                  if (angleY !== 0) {
                    const sin = SINE[angleY];
                    const cos = COSINE[angleY];
                    const temp = sin * this.verticesZ[v] + cos * this.verticesX[v] >> 16;
                    this.verticesZ[v] = cos * this.verticesZ[v] - sin * this.verticesX[v] >> 16;
                    this.verticesX[v] = temp;
                  }
                  this.verticesX[v] += _Model.animateOriginX;
                  this.verticesY[v] += _Model.animateOriginY;
                  this.verticesZ[v] += _Model.animateOriginZ;
                }
              }
            }
            break;
          case 3 /* SCALE */:
            for (const label of labels) {
              if (label < this.vertexLabels.length) {
                for (const v of this.vertexLabels[label]) {
                  this.verticesX[v] -= _Model.animateOriginX;
                  this.verticesY[v] -= _Model.animateOriginY;
                  this.verticesZ[v] -= _Model.animateOriginZ;
                  this.verticesX[v] = tx * this.verticesX[v] / 128 | 0;
                  this.verticesY[v] = ty * this.verticesY[v] / 128 | 0;
                  this.verticesZ[v] = tz * this.verticesZ[v] / 128 | 0;
                  this.verticesX[v] += _Model.animateOriginX;
                  this.verticesY[v] += _Model.animateOriginY;
                  this.verticesZ[v] += _Model.animateOriginZ;
                }
              }
            }
            break;
          case 5 /* ALPHA */:
            if (this.faceLabels && this.faceAlphas) {
              for (const label of labels) {
                if (label < this.faceLabels.length) {
                  for (const f of this.faceLabels[label]) {
                    let newAlpha = (this.faceAlphas[f] & 255) + tx * 8;
                    if (newAlpha < 0) {
                      newAlpha = 0;
                    } else if (newAlpha > 255) {
                      newAlpha = 255;
                    }
                    this.faceAlphas[f] = newAlpha;
                  }
                }
              }
            }
            break;
          case 7 /* LIGHT */:
            if (!this.faceLabels) {
              return;
            }
            for (const label of labels) {
              if (label >= this.faceLabels.length) {
                continue;
              }
              for (const f of this.faceLabels[label]) {
                const color = this.faceColors[f];
                let hue = color >> 10 & 63;
                let saturation = color >> 7 & 7;
                let lightness = color & 127;
                hue = hue + tx & 63;
                saturation += ty;
                if (saturation < 0) {
                  saturation = 0;
                } else if (saturation > 7) {
                  saturation = 7;
                }
                lightness += tz;
                if (lightness < 0) {
                  lightness = 0;
                } else if (lightness > 127) {
                  lightness = 127;
                }
                this.faceColors[f] = (hue << 10) + (saturation << 7) + lightness;
              }
              this.changedLight = true;
            }
            break;
        }
      }
      animateSkeletal(skeletalSeq, frame, options = {}) {
        const {
          masks,
          maskMatch = false,
          applyAlpha = !maskMatch,
          deferTransform = false
        } = options;
        const skeletalBase = skeletalSeq.skeletalBase;
        if (!skeletalBase) {
          if (!deferTransform && applyAlpha && skeletalSeq.hasAlphaTransform) {
            this.transformSkeletalAlpha(skeletalSeq, frame);
            this.invalidateBounds();
          }
          return;
        }
        skeletalBase.updateAnimMatrices(skeletalSeq, frame, masks, maskMatch);
        if (!deferTransform) {
          this.applySkeletalPoseInternal(skeletalSeq, skeletalBase, frame, applyAlpha);
        }
      }
      applySkeletalPose(skeletalSeq, frame, options = {}) {
        const { applyAlpha = true } = options;
        const skeletalBase = skeletalSeq.skeletalBase;
        if (!skeletalBase) {
          return;
        }
        this.applySkeletalPoseInternal(skeletalSeq, skeletalBase, frame, applyAlpha);
      }
      animateSkeletalComposite(baseSeq, baseFrame, composite = {}) {
        const { masks, overlay } = composite;
        if (!masks || masks.length === 0 || !overlay) {
          this.animateSkeletal(baseSeq, baseFrame);
          return;
        }
        if (overlay.seq.skeletalBase !== baseSeq.skeletalBase) {
          this.animateSkeletal(baseSeq, baseFrame);
          return;
        }
        this.animateSkeletal(baseSeq, baseFrame, {
          masks,
          maskMatch: false,
          applyAlpha: false,
          deferTransform: true
        });
        this.animateSkeletal(overlay.seq, overlay.frame, {
          masks,
          maskMatch: true,
          applyAlpha: false,
          deferTransform: true
        });
        this.applySkeletalPose(baseSeq, baseFrame, { applyAlpha: true });
      }
      transformSkeletal(skeletalBase, poseId, frame) {
        if (!this.animMayaGroups) {
          return;
        }
        for (let v = 0; v < this.verticesCount; v++) {
          const group = this.animMayaGroups[v];
          if (group && group.length !== 0) {
            const scalings = this.animMayaScales[v];
            for (let i = 0; i < 16; i++) {
              _Model.sketetalTransformMatrix[i] = 0;
            }
            for (let i = 0; i < group.length; i++) {
              const boneId = group[i];
              const bone = skeletalBase.getBone(boneId);
              if (bone) {
                const scale = scalings[i] / 255;
                import_gl_matrix.vec3.set(scaleVector, scale, scale, scale);
                import_gl_matrix.mat4.fromScaling(_Model.skeletalScalingMatrix, scaleVector);
                import_gl_matrix.mat4.mul(
                  _Model.skeletalBoneMatrix,
                  _Model.skeletalScalingMatrix,
                  bone.getFinalMatrix(poseId)
                );
                import_gl_matrix.mat4.add(
                  _Model.sketetalTransformMatrix,
                  _Model.sketetalTransformMatrix,
                  _Model.skeletalBoneMatrix
                );
              }
            }
            this.transformVertex(v, _Model.sketetalTransformMatrix);
          }
        }
      }
      transformVertex(v, m) {
        const vx = this.verticesX[v];
        const vy = -this.verticesY[v];
        const vz = -this.verticesZ[v];
        const scale = 1;
        this.verticesX[v] = Math.round(m[0] * vx + m[4] * vy + m[8] * vz + m[12] * scale);
        this.verticesY[v] = -Math.round(m[1] * vx + m[5] * vy + m[9] * vz + m[13] * scale);
        this.verticesZ[v] = -Math.round(m[2] * vx + m[6] * vy + m[10] * vz + m[14] * scale);
      }
      transformSkeletalAlpha(skeletalSeq, frame) {
        const base = skeletalSeq.base;
        for (let i = 0; i < base.count; i++) {
          const type = base.types[i];
          if (type === 5 /* ALPHA */ && skeletalSeq.curves && skeletalSeq.curves[i] && skeletalSeq.curves[i][0] && this.faceLabels && this.faceAlphas) {
            const curve = skeletalSeq.curves[i][0];
            const labels = base.labels[i];
            for (const l of labels) {
              if (l < this.faceLabels.length) {
                for (const f of this.faceLabels[l]) {
                  let newAlpha = (this.faceAlphas[f] & 255) + curve.getValue(frame) * 255;
                  if (newAlpha < 0) {
                    newAlpha = 0;
                  } else if (newAlpha > 255) {
                    newAlpha = 255;
                  }
                  this.faceAlphas[f] = newAlpha;
                }
              }
            }
          }
        }
      }
      applySkeletalPoseInternal(skeletalSeq, skeletalBase, frame, applyAlpha) {
        this.transformSkeletal(skeletalBase, skeletalSeq.poseId, frame);
        if (applyAlpha && skeletalSeq.hasAlphaTransform) {
          this.transformSkeletalAlpha(skeletalSeq, frame);
        }
        this.invalidateBounds();
      }
      postAnimate() {
        if (this.changedLight) {
          for (let i = 0; i < this.faceCount; i++) {
            const textureId = this.faceTextures ? this.faceTextures[i] : -1;
            if (textureId !== -1) {
              continue;
            }
            const color = this.faceColors[i] & 65535;
            if (this.faceColors3[i] === -1) {
              const i_617_ = this.faceColors1[i] & ~131071;
              this.faceColors1[i] = i_617_ | ModelData.adjustLightness(color, i_617_ >> 17);
            } else if (this.faceColors3[i] !== -2) {
              let c = this.faceColors1[i] & ~131071;
              this.faceColors1[i] = c | ModelData.adjustLightness(color, c >> 17);
              c = this.faceColors2[i] & ~131071;
              this.faceColors2[i] = c | ModelData.adjustLightness(color, c >> 17);
              c = this.faceColors3[i] & ~131071;
              this.faceColors3[i] = c | ModelData.adjustLightness(color, c >> 17);
            }
          }
          this.changedLight = false;
        }
        this.invalidateBounds();
      }
      // Linearly blend this model's vertex positions toward another model's positions.
      // Assumes both models share the same topology and vertex count.
      morphLerp(target, alpha) {
        const t = Math.max(0, Math.min(1, alpha));
        if (t === 0) return;
        const n = Math.min(this.verticesCount | 0, target.verticesCount | 0);
        const ax = this.verticesX, ay = this.verticesY, az = this.verticesZ;
        const bx = target.verticesX, by = target.verticesY, bz = target.verticesZ;
        for (let i = 0; i < n; i++) {
          ax[i] = ax[i] + (bx[i] - ax[i]) * t | 0;
          ay[i] = ay[i] + (by[i] - ay[i]) * t | 0;
          az[i] = az[i] + (bz[i] - az[i]) * t | 0;
        }
        this.invalidateBounds();
      }
    };
  }
});

// {TSPS_CHECKOUT}/client/rs/model/VertexNormal.ts
var VertexNormal;
var init_VertexNormal = __esm({
  "{TSPS_CHECKOUT}/client/rs/model/VertexNormal.ts"() {
    "use strict";
    VertexNormal = class _VertexNormal {
      constructor() {
        this.x = 0;
        this.y = 0;
        this.z = 0;
        this.magnitude = 0;
      }
      static copy(other) {
        const normal = new _VertexNormal();
        normal.x = other.x;
        normal.y = other.y;
        normal.z = other.z;
        normal.magnitude = other.magnitude;
        return normal;
      }
    };
  }
});

// {TSPS_CHECKOUT}/client/rs/model/ModelData.ts
var ModelData_exports = {};
__export(ModelData_exports, {
  ModelData: () => ModelData
});
var ModelData;
var init_ModelData = __esm({
  "{TSPS_CHECKOUT}/client/rs/model/ModelData.ts"() {
    "use strict";
    init_MathConstants();
    init_ByteBuffer();
    init_Entity();
    init_ColorUtil();
    init_FaceNormal();
    init_Model();
    init_TextureMapper();
    init_VertexNormal();
    ModelData = class _ModelData extends Entity {
      static {
        this.mergeModelNormalsCount = 0;
      }
      static {
        this.mergedNormalsModel0Cache = new Int32Array(1e4);
      }
      static {
        this.mergedNormalsModel1Cache = new Int32Array(1e4);
      }
      static {
        this.MERGE_NORMAL_Y_TOLERANCE = 2;
      }
      static merge(models, count) {
        const model = new _ModelData();
        model.merge(models, count);
        return model;
      }
      static decode(data) {
        const model = new _ModelData();
        model.decode(data);
        return model;
      }
      static decodeLegacy(loader, meta) {
        const model = new _ModelData();
        model.decodeLegacy(loader, meta);
        return model;
      }
      static copyFrom(model, shallowIndices, shallowVertices, shallowColors, shallowTextures) {
        const copy = new _ModelData();
        copy.copyFrom(model, shallowIndices, shallowVertices, shallowColors, shallowTextures);
        return copy;
      }
      static adjustLightness(hsl, lightness) {
        return blendLight(hsl, lightness);
      }
      static clampLightness(lightness) {
        if (lightness < 2) {
          lightness = 2;
        } else if (lightness > 126) {
          lightness = 126;
        }
        return lightness | 0;
      }
      static mergeNormals(model0, model1, offsetX, offsetY, offsetZ, hideOccludedFaces) {
        model0.calculateBounds();
        model0.calculateVertexNormals();
        model1.calculateBounds();
        model1.calculateVertexNormals();
        if (!model0.normals || !model1.normals) {
          return;
        }
        _ModelData.mergeModelNormalsCount++;
        const verticesY0 = model0.contourVerticesY || model0.verticesY;
        const verticesY1 = model1.contourVerticesY || model1.verticesY;
        let mergedCount = 0;
        if (model0.maxX - offsetX < model1.minX || model0.minX - offsetX > model1.maxX || model0.maxZ - offsetZ < model1.minZ || model0.minZ - offsetZ > model1.maxZ || model0.minY - offsetY > model1.minHeight + _ModelData.MERGE_NORMAL_Y_TOLERANCE) {
          return;
        }
        let vertexMap;
        const buildVertexMap = () => {
          const map = /* @__PURE__ */ new Map();
          for (let v1 = 0; v1 < model1.usedVertexCount; v1++) {
            if (model1.normals[v1].magnitude === 0) {
              continue;
            }
            const key = model1.verticesX[v1] * 2147483648 + model1.verticesZ[v1];
            let indices = map.get(key);
            if (!indices) {
              indices = [];
              map.set(key, indices);
            }
            indices.push(v1);
          }
          return map;
        };
        for (let v0 = 0; v0 < model0.usedVertexCount; v0++) {
          const normal0 = model0.normals[v0];
          if (normal0.magnitude === 0) {
            continue;
          }
          const y = verticesY0[v0] - offsetY;
          if (y > model1.minHeight + _ModelData.MERGE_NORMAL_Y_TOLERANCE) {
            continue;
          }
          const x = model0.verticesX[v0] - offsetX;
          if (x < model1.minX || x > model1.maxX) {
            continue;
          }
          const z = model0.verticesZ[v0] - offsetZ;
          if (z < model1.minZ || z > model1.maxZ) {
            continue;
          }
          vertexMap ??= buildVertexMap();
          const matchingIndices = vertexMap.get(x * 2147483648 + z);
          if (!matchingIndices) {
            continue;
          }
          for (const v1 of matchingIndices) {
            if (Math.abs(y - verticesY1[v1]) > _ModelData.MERGE_NORMAL_Y_TOLERANCE) {
              continue;
            }
            const normal1 = model1.normals[v1];
            if (!model0.mergedNormals) {
              model0.mergedNormals = new Array(model0.usedVertexCount);
            }
            if (!model1.mergedNormals) {
              model1.mergedNormals = new Array(model1.usedVertexCount);
            }
            let mergedNormal0 = model0.mergedNormals[v0];
            if (!mergedNormal0) {
              mergedNormal0 = model0.mergedNormals[v0] = VertexNormal.copy(normal0);
            }
            let mergedNormal1 = model1.mergedNormals[v1];
            if (!mergedNormal1) {
              mergedNormal1 = model1.mergedNormals[v1] = VertexNormal.copy(normal1);
            }
            mergedNormal0.x += normal1.x;
            mergedNormal0.y += normal1.y;
            mergedNormal0.z += normal1.z;
            mergedNormal0.magnitude += normal1.magnitude;
            mergedNormal1.x += normal0.x;
            mergedNormal1.y += normal0.y;
            mergedNormal1.z += normal0.z;
            mergedNormal1.magnitude += normal0.magnitude;
            mergedCount++;
            _ModelData.mergedNormalsModel0Cache[v0] = _ModelData.mergeModelNormalsCount;
            _ModelData.mergedNormalsModel1Cache[v1] = _ModelData.mergeModelNormalsCount;
          }
        }
        if (mergedCount >= 3 && hideOccludedFaces) {
          for (let i = 0; i < model0.faceCount; i++) {
            if (_ModelData.mergedNormalsModel0Cache[model0.indices1[i]] === _ModelData.mergeModelNormalsCount && _ModelData.mergedNormalsModel0Cache[model0.indices2[i]] === _ModelData.mergeModelNormalsCount && _ModelData.mergedNormalsModel0Cache[model0.indices3[i]] === _ModelData.mergeModelNormalsCount) {
              if (!model0.faceRenderTypes) {
                model0.faceRenderTypes = new Int8Array(model0.faceCount);
              }
              model0.faceRenderTypes[i] = 2;
            }
          }
          for (let i = 0; i < model1.faceCount; i++) {
            if (_ModelData.mergedNormalsModel1Cache[model1.indices1[i]] === _ModelData.mergeModelNormalsCount && _ModelData.mergedNormalsModel1Cache[model1.indices2[i]] === _ModelData.mergeModelNormalsCount && _ModelData.mergedNormalsModel1Cache[model1.indices3[i]] === _ModelData.mergeModelNormalsCount) {
              if (!model1.faceRenderTypes) {
                model1.faceRenderTypes = new Int8Array(model1.faceCount);
              }
              model1.faceRenderTypes[i] = 2;
            }
          }
        }
      }
      constructor() {
        super();
        this.version = -1;
        this.verticesCount = 0;
        this.usedVertexCount = 0;
        this.faceCount = 0;
        this.priority = 0;
        this.isBoundsCalculated = false;
      }
      canMergeNormals() {
        return true;
      }
      mergeNormals(entity, offsetX, offsetY, offsetZ, hideOccluded) {
        if (!(entity instanceof _ModelData)) {
          return;
        }
        _ModelData.mergeNormals(this, entity, offsetX, offsetY, offsetZ, hideOccluded);
      }
      merge(models, count) {
        this.version = 12;
        this.verticesCount = 0;
        this.faceCount = 0;
        this.textureFaceCount = 0;
        this.priority = -1;
        let hasRenderTypes = false;
        let hasRenderPriorities = false;
        let hasAlphas = false;
        let hasFaceSkins = false;
        let hasTextures = false;
        let hasTextureCoords = false;
        let hasMayaGroups = false;
        for (let i = 0; i < count; i++) {
          const model = models[i];
          if (model) {
            this.verticesCount += model.verticesCount;
            this.faceCount += model.faceCount;
            this.textureFaceCount += model.textureFaceCount;
            if (model.faceRenderPriorities) {
              hasRenderPriorities = true;
            } else {
              if (this.priority === -1) {
                this.priority = model.priority;
              }
              if (this.priority !== model.priority) {
                hasRenderPriorities = true;
              }
            }
            hasRenderTypes ||= !!model.faceRenderTypes;
            hasAlphas ||= !!model.faceAlphas;
            hasFaceSkins ||= !!model.faceSkins;
            hasTextures ||= !!model.faceTextures;
            hasTextureCoords ||= !!model.textureCoords;
            hasMayaGroups ||= !!model.animMayaGroups;
          }
        }
        this.verticesX = new Int32Array(this.verticesCount);
        this.verticesY = new Int32Array(this.verticesCount);
        this.verticesZ = new Int32Array(this.verticesCount);
        this.vertexSkins = new Int32Array(this.verticesCount);
        this.indices1 = new Int32Array(this.faceCount);
        this.indices2 = new Int32Array(this.faceCount);
        this.indices3 = new Int32Array(this.faceCount);
        if (hasRenderTypes) {
          this.faceRenderTypes = new Int8Array(this.faceCount);
        }
        if (hasRenderPriorities) {
          this.faceRenderPriorities = new Int8Array(this.faceCount);
        }
        if (hasAlphas) {
          this.faceAlphas = new Int8Array(this.faceCount);
        }
        if (hasFaceSkins) {
          this.faceSkins = new Int32Array(this.faceCount);
        }
        if (hasTextures) {
          this.faceTextures = new Int16Array(this.faceCount);
        }
        if (hasTextureCoords) {
          this.textureCoords = new Int8Array(this.faceCount);
        }
        if (hasMayaGroups) {
          this.animMayaGroups = new Array(this.verticesCount);
          this.animMayaScales = new Array(this.verticesCount);
        }
        this.faceColors = new Uint16Array(this.faceCount);
        if (this.textureFaceCount > 0) {
          this.textureRenderTypes = new Int8Array(this.textureFaceCount);
          this.textureMappingP = new Int16Array(this.textureFaceCount);
          this.textureMappingM = new Int16Array(this.textureFaceCount);
          this.textureMappingN = new Int16Array(this.textureFaceCount);
          this.textureScaleX = new Int32Array(this.textureFaceCount);
          this.textureScaleY = new Int32Array(this.textureFaceCount);
          this.textureScaleZ = new Int32Array(this.textureFaceCount);
          this.textureRotation = new Int8Array(this.textureFaceCount);
          this.textureDirection = new Int8Array(this.textureFaceCount);
          this.textureSpeed = new Int32Array(this.textureFaceCount);
          this.textureTransU = new Int32Array(this.textureFaceCount);
          this.textureTransV = new Int32Array(this.textureFaceCount);
        }
        this.verticesCount = 0;
        this.faceCount = 0;
        this.textureFaceCount = 0;
        for (let i = 0; i < count; i++) {
          const model = models[i];
          if (!model) {
            continue;
          }
          for (let f = 0; f < model.faceCount; f++) {
            if (hasRenderTypes && model.faceRenderTypes && this.faceRenderTypes) {
              this.faceRenderTypes[this.faceCount] = model.faceRenderTypes[f];
            }
            if (hasRenderPriorities) {
              if (model.faceRenderPriorities) {
                this.faceRenderPriorities[this.faceCount] = model.faceRenderPriorities[f];
              } else {
                this.faceRenderPriorities[this.faceCount] = model.priority;
              }
            }
            if (hasAlphas && model.faceAlphas) {
              this.faceAlphas[this.faceCount] = model.faceAlphas[f];
            }
            if (hasFaceSkins && this.faceSkins) {
              if (model.faceSkins) {
                this.faceSkins[this.faceCount] = model.faceSkins[f];
              } else {
                this.faceSkins[this.faceCount] = -1;
              }
            }
            if (hasTextures && this.faceTextures) {
              if (model.faceTextures) {
                this.faceTextures[this.faceCount] = model.faceTextures[f];
              } else {
                this.faceTextures[this.faceCount] = -1;
              }
            }
            if (hasTextureCoords && this.textureCoords) {
              if (model.textureCoords && model.textureCoords[f] !== -1) {
                this.textureCoords[this.faceCount] = this.textureFaceCount + model.textureCoords[f];
              } else {
                this.textureCoords[this.faceCount] = -1;
              }
            }
            this.faceColors[this.faceCount] = model.faceColors[f];
            this.indices1[this.faceCount] = this.copyVertex(model, model.indices1[f]);
            this.indices2[this.faceCount] = this.copyVertex(model, model.indices2[f]);
            this.indices3[this.faceCount] = this.copyVertex(model, model.indices3[f]);
            this.faceCount++;
          }
          for (let f = 0; f < model.textureFaceCount; f++) {
            const type = this.textureRenderTypes[this.textureFaceCount] = model.textureRenderTypes[f];
            if (type === 0) {
              this.textureMappingP[this.textureFaceCount] = this.copyVertex(
                model,
                model.textureMappingP[f]
              );
              this.textureMappingM[this.textureFaceCount] = this.copyVertex(
                model,
                model.textureMappingM[f]
              );
              this.textureMappingN[this.textureFaceCount] = this.copyVertex(
                model,
                model.textureMappingN[f]
              );
            }
            if (type >= 1 && type <= 3) {
              this.textureMappingP[this.textureFaceCount] = model.textureMappingP[f];
              this.textureMappingM[this.textureFaceCount] = model.textureMappingM[f];
              this.textureMappingN[this.textureFaceCount] = model.textureMappingN[f];
              this.textureScaleX[this.textureFaceCount] = model.textureScaleX[f];
              this.textureScaleY[this.textureFaceCount] = model.textureScaleY[f];
              this.textureScaleZ[this.textureFaceCount] = model.textureScaleZ[f];
              this.textureRotation[this.textureFaceCount] = model.textureRotation[f];
              this.textureDirection[this.textureFaceCount] = model.textureDirection[f];
              this.textureSpeed[this.textureFaceCount] = model.textureSpeed[f];
            }
            if (type === 2) {
              this.textureTransU[this.textureFaceCount] = model.textureTransU[f];
              this.textureTransV[this.textureFaceCount] = model.textureTransV[f];
            }
            this.textureFaceCount++;
          }
        }
        this.usedVertexCount = this.verticesCount;
      }
      copyVertex(model, index) {
        let newVertexCount = -1;
        const vertX = model.verticesX[index];
        const vertY = model.verticesY[index];
        const vertZ = model.verticesZ[index];
        for (let i = 0; i < this.verticesCount; i++) {
          if (vertX === this.verticesX[i] && vertY === this.verticesY[i] && vertZ === this.verticesZ[i]) {
            newVertexCount = i;
            break;
          }
        }
        if (newVertexCount === -1) {
          this.verticesX[this.verticesCount] = vertX;
          this.verticesY[this.verticesCount] = vertY;
          this.verticesZ[this.verticesCount] = vertZ;
          if (model.vertexSkins && this.vertexSkins) {
            this.vertexSkins[this.verticesCount] = model.vertexSkins[index];
          } else if (this.vertexSkins) {
            this.vertexSkins[this.verticesCount] = -1;
          }
          if (model.animMayaGroups) {
            this.animMayaGroups[this.verticesCount] = model.animMayaGroups[index];
            this.animMayaScales[this.verticesCount] = model.animMayaScales[index];
          }
          newVertexCount = this.verticesCount++;
        }
        return newVertexCount;
      }
      decode(data) {
        if (data[data.length - 1] === -3 && data[data.length - 2] === -1) {
          this.decodeV3(data);
          this.usedVertexCount = this.verticesCount;
        } else if (data[data.length - 1] === -2 && data[data.length - 2] === -1) {
          this.decodeV2(data);
          this.usedVertexCount = this.verticesCount;
        } else if (data[data.length - 1] === -1 && data[data.length - 2] === -1) {
          this.decodeV1(data);
        } else {
          this.decodeOld(data);
        }
      }
      decodeV3(data) {
        this.version = 3;
        const buf1 = new ByteBuffer(data);
        const buf2 = new ByteBuffer(data);
        const buf3 = new ByteBuffer(data);
        const buf4 = new ByteBuffer(data);
        const buf5 = new ByteBuffer(data);
        const buf6 = new ByteBuffer(data);
        const buf7 = new ByteBuffer(data);
        buf1.offset = data.length - 26;
        const vertexCount = buf1.readUnsignedShort();
        const faceCount = buf1.readUnsignedShort();
        const texTriangleCount = buf1.readUnsignedByte();
        const var12 = buf1.readUnsignedByte();
        const var13 = buf1.readUnsignedByte();
        const var14 = buf1.readUnsignedByte();
        const var15 = buf1.readUnsignedByte();
        const var16 = buf1.readUnsignedByte();
        const var17 = buf1.readUnsignedByte();
        const hasMayaGroups = buf1.readUnsignedByte();
        const var19 = buf1.readUnsignedShort();
        const var20 = buf1.readUnsignedShort();
        const var21 = buf1.readUnsignedShort();
        const var22 = buf1.readUnsignedShort();
        const var23 = buf1.readUnsignedShort();
        const var24 = buf1.readUnsignedShort();
        let simpleTextureFaceCount = 0;
        let complexTextureFaceCount = 0;
        let cubeTextureFaceCount = 0;
        if (texTriangleCount > 0) {
          this.textureRenderTypes = new Int8Array(texTriangleCount);
          buf1.offset = 0;
          for (let i = 0; i < texTriangleCount; i++) {
            const type = this.textureRenderTypes[i] = buf1.readByte();
            if (type === 0) {
              simpleTextureFaceCount++;
            }
            if (type >= 1 && type <= 3) {
              complexTextureFaceCount++;
            }
            if (type === 2) {
              cubeTextureFaceCount++;
            }
          }
        }
        let var28 = texTriangleCount + vertexCount;
        const var30 = var28;
        if (var12 === 1) {
          var28 += faceCount;
        }
        const var31 = var28;
        var28 += faceCount;
        const var32 = var28;
        if (var13 === 255) {
          var28 += faceCount;
        }
        const var33 = var28;
        if (var15 === 1) {
          var28 += faceCount;
        }
        const var34 = var28;
        var28 += var24;
        const var35 = var28;
        if (var14 === 1) {
          var28 += faceCount;
        }
        const var36 = var28;
        var28 += var22;
        const var37 = var28;
        if (var16 === 1) {
          var28 += faceCount * 2;
        }
        const var38 = var28;
        var28 += var23;
        const var39 = var28;
        var28 += faceCount * 2;
        const var40 = var28;
        var28 += var19;
        const var41 = var28;
        var28 += var20;
        const var42 = var28;
        var28 += var21;
        const var43 = var28;
        var28 += simpleTextureFaceCount * 6;
        const var44 = var28;
        var28 += complexTextureFaceCount * 6;
        const var45 = var28;
        var28 += complexTextureFaceCount * 6;
        const var46 = var28;
        var28 += complexTextureFaceCount * 2;
        const var47 = var28;
        var28 += complexTextureFaceCount;
        const var48 = var28;
        var28 += complexTextureFaceCount * 2 + cubeTextureFaceCount * 2;
        this.verticesCount = vertexCount;
        this.faceCount = faceCount;
        this.textureFaceCount = texTriangleCount;
        this.verticesX = new Int32Array(vertexCount);
        this.verticesY = new Int32Array(vertexCount);
        this.verticesZ = new Int32Array(vertexCount);
        this.indices1 = new Int32Array(faceCount);
        this.indices2 = new Int32Array(faceCount);
        this.indices3 = new Int32Array(faceCount);
        if (var17 === 1) {
          this.vertexSkins = new Int32Array(vertexCount);
        }
        if (var12 === 1) {
          this.faceRenderTypes = new Int8Array(faceCount);
        }
        if (var13 === 255) {
          this.faceRenderPriorities = new Int8Array(faceCount);
        } else {
          this.priority = var13;
        }
        if (var14 === 1) {
          this.faceAlphas = new Int8Array(faceCount);
        }
        if (var15 === 1) {
          this.faceSkins = new Int32Array(faceCount);
        }
        if (var16 === 1) {
          this.faceTextures = new Int16Array(faceCount);
        }
        if (var16 === 1 && texTriangleCount > 0) {
          this.textureCoords = new Int8Array(faceCount);
        }
        if (hasMayaGroups === 1) {
          this.animMayaGroups = new Array(vertexCount);
          this.animMayaScales = new Array(vertexCount);
        }
        this.faceColors = new Uint16Array(faceCount);
        if (texTriangleCount > 0) {
          this.textureMappingP = new Int16Array(texTriangleCount);
          this.textureMappingM = new Int16Array(texTriangleCount);
          this.textureMappingN = new Int16Array(texTriangleCount);
          if (complexTextureFaceCount > 0) {
            this.textureScaleX = new Int32Array(complexTextureFaceCount);
            this.textureScaleY = new Int32Array(complexTextureFaceCount);
            this.textureScaleZ = new Int32Array(complexTextureFaceCount);
            this.textureRotation = new Int8Array(complexTextureFaceCount);
            this.textureDirection = new Int8Array(complexTextureFaceCount);
            this.textureSpeed = new Int32Array(complexTextureFaceCount);
          }
          if (cubeTextureFaceCount > 0) {
            this.textureTransU = new Int32Array(cubeTextureFaceCount);
            this.textureTransV = new Int32Array(cubeTextureFaceCount);
          }
        }
        buf1.offset = texTriangleCount;
        buf2.offset = var40;
        buf3.offset = var41;
        buf4.offset = var42;
        buf5.offset = var34;
        let lastVertX = 0;
        let lastVertY = 0;
        let lastVertZ = 0;
        for (let i = 0; i < vertexCount; i++) {
          const flag = buf1.readUnsignedByte();
          let deltaVertX = 0;
          if ((flag & 1) !== 0) {
            deltaVertX = buf2.readSmart2();
          }
          let deltaVertY = 0;
          if ((flag & 2) !== 0) {
            deltaVertY = buf3.readSmart2();
          }
          let deltaVertZ = 0;
          if ((flag & 4) !== 0) {
            deltaVertZ = buf4.readSmart2();
          }
          this.verticesX[i] = lastVertX + deltaVertX;
          this.verticesY[i] = lastVertY + deltaVertY;
          this.verticesZ[i] = lastVertZ + deltaVertZ;
          lastVertX = this.verticesX[i];
          lastVertY = this.verticesY[i];
          lastVertZ = this.verticesZ[i];
          if (var17 === 1 && this.vertexSkins) {
            this.vertexSkins[i] = buf5.readUnsignedByte();
          }
        }
        if (hasMayaGroups === 1) {
          for (let i = 0; i < vertexCount; i++) {
            const var542 = buf5.readUnsignedByte();
            this.animMayaGroups[i] = new Int32Array(var542);
            this.animMayaScales[i] = new Int32Array(var542);
            for (let j = 0; j < var542; j++) {
              this.animMayaGroups[i][j] = buf5.readUnsignedByte();
              this.animMayaScales[i][j] = buf5.readUnsignedByte();
            }
          }
        }
        buf1.offset = var39;
        buf2.offset = var30;
        buf3.offset = var32;
        buf4.offset = var35;
        buf5.offset = var33;
        buf6.offset = var37;
        buf7.offset = var38;
        for (let i = 0; i < faceCount; i++) {
          this.faceColors[i] = buf1.readUnsignedShort();
          if (var12 === 1 && this.faceRenderTypes) {
            this.faceRenderTypes[i] = buf2.readByte();
          }
          if (var13 === 255) {
            this.faceRenderPriorities[i] = buf3.readByte();
          }
          if (var14 === 1) {
            this.faceAlphas[i] = buf4.readByte();
          }
          if (var15 === 1 && this.faceSkins) {
            this.faceSkins[i] = buf5.readUnsignedByte();
          }
          if (var16 === 1 && this.faceTextures) {
            this.faceTextures[i] = buf6.readUnsignedShort() - 1;
          }
          if (this.textureCoords && this.faceTextures && this.faceTextures[i] !== -1) {
            this.textureCoords[i] = buf7.readUnsignedByte() - 1;
          }
        }
        buf1.offset = var36;
        buf2.offset = var31;
        let var53 = 0;
        let var54 = 0;
        let var55 = 0;
        let var56 = 0;
        for (let i = 0; i < faceCount; i++) {
          const type = buf2.readUnsignedByte();
          if (type === 1) {
            var53 = buf1.readSmart2() + var56;
            var54 = buf1.readSmart2() + var53;
            var55 = buf1.readSmart2() + var54;
            var56 = var55;
            this.indices1[i] = var53;
            this.indices2[i] = var54;
            this.indices3[i] = var55;
          }
          if (type === 2) {
            var54 = var55;
            var55 = buf1.readSmart2() + var56;
            var56 = var55;
            this.indices1[i] = var53;
            this.indices2[i] = var54;
            this.indices3[i] = var55;
          }
          if (type === 3) {
            var53 = var55;
            var55 = buf1.readSmart2() + var56;
            var56 = var55;
            this.indices1[i] = var53;
            this.indices2[i] = var54;
            this.indices3[i] = var55;
          }
          if (type === 4) {
            const var59 = var53;
            var53 = var54;
            var54 = var59;
            var55 = buf1.readSmart2() + var56;
            var56 = var55;
            this.indices1[i] = var53;
            this.indices2[i] = var59;
            this.indices3[i] = var55;
          }
        }
        buf1.offset = var43;
        buf2.offset = var44;
        buf3.offset = var45;
        buf4.offset = var46;
        buf5.offset = var47;
        buf6.offset = var48;
        for (let i = 0; i < texTriangleCount; i++) {
          const type = this.textureRenderTypes[i] & 255;
          if (type === 0) {
            this.textureMappingP[i] = buf1.readUnsignedShort();
            this.textureMappingM[i] = buf1.readUnsignedShort();
            this.textureMappingN[i] = buf1.readUnsignedShort();
          }
        }
        buf1.offset = var28;
        const var57 = buf1.readUnsignedByte();
        if (var57 !== 0) {
          buf1.readUnsignedShort();
          buf1.readUnsignedShort();
          buf1.readUnsignedShort();
          buf1.readInt();
        }
      }
      decodeV2(data) {
        this.version = 2;
        let var2 = false;
        let var3 = false;
        const buf1 = new ByteBuffer(data);
        const buf2 = new ByteBuffer(data);
        const buf3 = new ByteBuffer(data);
        const buf4 = new ByteBuffer(data);
        const buf5 = new ByteBuffer(data);
        buf1.offset = data.length - 23;
        const vertexCount = buf1.readUnsignedShort();
        const faceCount = buf1.readUnsignedShort();
        const texTriangleCount = buf1.readUnsignedByte();
        const var12 = buf1.readUnsignedByte();
        const var13 = buf1.readUnsignedByte();
        const var14 = buf1.readUnsignedByte();
        const var15 = buf1.readUnsignedByte();
        const hasVertexSkins = buf1.readUnsignedByte();
        const hasMayaGroups = buf1.readUnsignedByte();
        const var18 = buf1.readUnsignedShort();
        const var19 = buf1.readUnsignedShort();
        const var20 = buf1.readUnsignedShort();
        const var21 = buf1.readUnsignedShort();
        const var22 = buf1.readUnsignedShort();
        let var23 = 0;
        let var47 = var23 + vertexCount;
        const var25 = var47;
        var47 += faceCount;
        const var26 = var47;
        if (var13 === 255) {
          var47 += faceCount;
        }
        const var27 = var47;
        if (var15 === 1) {
          var47 += faceCount;
        }
        const var28 = var47;
        if (var12 === 1) {
          var47 += faceCount;
        }
        const var29 = var47;
        var47 += var22;
        const var30 = var47;
        if (var14 === 1) {
          var47 += faceCount;
        }
        const var31 = var47;
        var47 += var21;
        const var32 = var47;
        var47 += faceCount * 2;
        const var33 = var47;
        var47 += texTriangleCount * 6;
        const var34 = var47;
        var47 += var18;
        const var35 = var47;
        var47 += var19;
        this.verticesCount = vertexCount;
        this.faceCount = faceCount;
        this.textureFaceCount = texTriangleCount;
        this.verticesX = new Int32Array(vertexCount);
        this.verticesY = new Int32Array(vertexCount);
        this.verticesZ = new Int32Array(vertexCount);
        this.indices1 = new Int32Array(faceCount);
        this.indices2 = new Int32Array(faceCount);
        this.indices3 = new Int32Array(faceCount);
        if (texTriangleCount > 0) {
          this.textureRenderTypes = new Int8Array(texTriangleCount);
          this.textureMappingP = new Int16Array(texTriangleCount);
          this.textureMappingM = new Int16Array(texTriangleCount);
          this.textureMappingN = new Int16Array(texTriangleCount);
        }
        if (hasVertexSkins === 1) {
          this.vertexSkins = new Int32Array(vertexCount);
        }
        if (var12 === 1) {
          this.faceRenderTypes = new Int8Array(faceCount);
          this.textureCoords = new Int8Array(faceCount);
          this.faceTextures = new Int16Array(faceCount);
        }
        if (var13 === 255) {
          this.faceRenderPriorities = new Int8Array(faceCount);
        } else {
          this.priority = var13;
        }
        if (var14 === 1) {
          this.faceAlphas = new Int8Array(faceCount);
        }
        if (var15 === 1) {
          this.faceSkins = new Int32Array(faceCount);
        }
        if (hasMayaGroups === 1) {
          this.animMayaGroups = new Array(vertexCount);
          this.animMayaScales = new Array(vertexCount);
        }
        this.faceColors = new Uint16Array(faceCount);
        buf1.offset = var23;
        buf2.offset = var34;
        buf3.offset = var35;
        buf4.offset = var47;
        buf5.offset = var29;
        let lastVertX = 0;
        let lastVertY = 0;
        let lastVertZ = 0;
        for (let i = 0; i < vertexCount; i++) {
          const flag = buf1.readUnsignedByte();
          let deltaVertX = 0;
          if ((flag & 1) !== 0) {
            deltaVertX = buf2.readSmart2();
          }
          let deltaVertY = 0;
          if ((flag & 2) !== 0) {
            deltaVertY = buf3.readSmart2();
          }
          let deltaVertZ = 0;
          if ((flag & 4) !== 0) {
            deltaVertZ = buf4.readSmart2();
          }
          this.verticesX[i] = lastVertX + deltaVertX;
          this.verticesY[i] = lastVertY + deltaVertY;
          this.verticesZ[i] = lastVertZ + deltaVertZ;
          lastVertX = this.verticesX[i];
          lastVertY = this.verticesY[i];
          lastVertZ = this.verticesZ[i];
          if (hasVertexSkins === 1 && this.vertexSkins) {
            this.vertexSkins[i] = buf5.readUnsignedByte();
          }
        }
        if (hasMayaGroups === 1) {
          for (let i = 0; i < vertexCount; i++) {
            const var412 = buf5.readUnsignedByte();
            this.animMayaGroups[i] = new Int32Array(var412);
            this.animMayaScales[i] = new Int32Array(var412);
            for (let j = 0; j < var412; j++) {
              this.animMayaGroups[i][j] = buf5.readUnsignedByte();
              this.animMayaScales[i][j] = buf5.readUnsignedByte();
            }
          }
        }
        buf1.offset = var32;
        buf2.offset = var28;
        buf3.offset = var26;
        buf4.offset = var30;
        buf5.offset = var27;
        for (let i = 0; i < faceCount; i++) {
          this.faceColors[i] = buf1.readUnsignedShort();
          if (var12 === 1 && this.faceRenderTypes && this.textureCoords && this.faceTextures) {
            const var412 = buf2.readUnsignedByte();
            if ((var412 & 1) === 1) {
              this.faceRenderTypes[i] = 1;
              var2 = true;
            } else {
              this.faceRenderTypes[i] = 0;
            }
            if ((var412 & 2) === 2) {
              this.textureCoords[i] = var412 >> 2;
              this.faceTextures[i] = this.faceColors[i];
              this.faceColors[i] = 127;
              if (this.faceTextures[i] !== -1) {
                var3 = true;
              }
            } else {
              this.textureCoords[i] = -1;
              this.faceTextures[i] = -1;
            }
          }
          if (var13 === 255) {
            this.faceRenderPriorities[i] = buf3.readByte();
          }
          if (var14 === 1) {
            this.faceAlphas[i] = buf4.readByte();
          }
          if (var15 === 1 && this.faceSkins) {
            this.faceSkins[i] = buf5.readUnsignedByte();
          }
        }
        buf1.offset = var31;
        buf2.offset = var25;
        let var40 = 0;
        let var41 = 0;
        let var42 = 0;
        let var43 = 0;
        for (let i = 0; i < faceCount; i++) {
          const var45 = buf2.readUnsignedByte();
          if (var45 === 1) {
            var40 = buf1.readSmart2() + var43;
            var41 = buf1.readSmart2() + var40;
            var42 = buf1.readSmart2() + var41;
            var43 = var42;
            this.indices1[i] = var40;
            this.indices2[i] = var41;
            this.indices3[i] = var42;
          }
          if (var45 === 2) {
            var41 = var42;
            var42 = buf1.readSmart2() + var43;
            var43 = var42;
            this.indices1[i] = var40;
            this.indices2[i] = var41;
            this.indices3[i] = var42;
          }
          if (var45 === 3) {
            var40 = var42;
            var42 = buf1.readSmart2() + var43;
            var43 = var42;
            this.indices1[i] = var40;
            this.indices2[i] = var41;
            this.indices3[i] = var42;
          }
          if (var45 === 4) {
            const var46 = var40;
            var40 = var41;
            var41 = var46;
            var42 = buf1.readSmart2() + var43;
            var43 = var42;
            this.indices1[i] = var40;
            this.indices2[i] = var46;
            this.indices3[i] = var42;
          }
        }
        buf1.offset = var33;
        for (let i = 0; i < texTriangleCount; i++) {
          this.textureRenderTypes[i] = 0;
          this.textureMappingP[i] = buf1.readUnsignedShort();
          this.textureMappingM[i] = buf1.readUnsignedShort();
          this.textureMappingN[i] = buf1.readUnsignedShort();
        }
        if (this.textureCoords) {
          let var48 = false;
          for (let i = 0; i < faceCount; i++) {
            const coord = this.textureCoords[i] & 255;
            if (coord !== 255) {
              if (this.indices1[i] === (this.textureMappingP[coord] & 65535) && this.indices2[i] === (this.textureMappingM[coord] & 65535) && this.indices3[i] === (this.textureMappingN[coord] & 65535)) {
                this.textureCoords[i] = -1;
              } else {
                var48 = true;
              }
            }
          }
          if (!var48) {
            this.textureCoords = void 0;
          }
        }
        if (!var3) {
          this.faceTextures = void 0;
        }
        if (!var2) {
          this.faceRenderTypes = void 0;
        }
      }
      scaleDown(n) {
        for (let i = 0; i < this.verticesCount; i++) {
          this.verticesX[i] >>= n;
          this.verticesY[i] >>= n;
          this.verticesZ[i] >>= n;
        }
        if (this.textureFaceCount > 0 && this.textureScaleX) {
          for (let i = 0; i < this.textureFaceCount; i++) {
            this.textureScaleX[i] >>= n;
            this.textureScaleY[i] >>= n;
            if (this.textureRenderTypes[i] !== 1) {
              this.textureScaleZ[i] >>= n;
            }
          }
        }
      }
      decodeV1(data) {
        this.version = 1;
        const buf1 = new ByteBuffer(data);
        const buf2 = new ByteBuffer(data);
        const buf3 = new ByteBuffer(data);
        const buf4 = new ByteBuffer(data);
        const buf5 = new ByteBuffer(data);
        const buf6 = new ByteBuffer(data);
        const buf7 = new ByteBuffer(data);
        buf1.offset = data.length - 23;
        const vertexCount = buf1.readUnsignedShort();
        const faceCount = buf1.readUnsignedShort();
        const texFaceCount = buf1.readUnsignedByte();
        const flags = buf1.readUnsignedByte();
        const hasFaceRenderTypes = (flags & 1) === 1;
        const hasParticles = (flags & 2) === 2;
        const hasBillboards = (flags & 4) === 4;
        const hasVersion = (flags & 8) === 8;
        if (hasVersion) {
          buf1.offset -= 7;
          this.version = buf1.readUnsignedByte();
          buf1.offset += 6;
        }
        const modelPriority = buf1.readUnsignedByte();
        const hasFaceAlpha = buf1.readUnsignedByte();
        const hasFaceSkins = buf1.readUnsignedByte();
        const hasFaceTextures = buf1.readUnsignedByte();
        const hasVertexSkins = buf1.readUnsignedByte();
        const modelVerticesX = buf1.readUnsignedShort();
        const modelVerticesY = buf1.readUnsignedShort();
        const modelVerticesZ = buf1.readUnsignedShort();
        const faceIndices = buf1.readUnsignedShort();
        const textureIndices = buf1.readUnsignedShort();
        let simpleTextureFaceCount = 0;
        let complexTextureFaceCount = 0;
        let cubeTextureFaceCount = 0;
        if (texFaceCount > 0) {
          this.textureRenderTypes = new Int8Array(texFaceCount);
          buf1.offset = 0;
          for (let i = 0; i < texFaceCount; i++) {
            const type = this.textureRenderTypes[i] = buf1.readByte();
            if (type === 0) {
              simpleTextureFaceCount++;
            }
            if (type >= 1 && type <= 3) {
              complexTextureFaceCount++;
            }
            if (type === 2) {
              cubeTextureFaceCount++;
            }
          }
        }
        let offset = texFaceCount + vertexCount;
        const vertexFlagsOffset = offset;
        if (hasFaceRenderTypes) {
          offset += faceCount;
        }
        const faceCompressTypeOffset = offset;
        offset += faceCount;
        const facePrioritiesOffset = offset;
        if (modelPriority === 255) {
          offset += faceCount;
        }
        const faceSkinsOffset = offset;
        if (hasFaceSkins === 1) {
          offset += faceCount;
        }
        const vertexSkinsOffset = offset;
        if (hasVertexSkins === 1) {
          offset += vertexCount;
        }
        const faceAlphasOffset = offset;
        if (hasFaceAlpha === 1) {
          offset += faceCount;
        }
        const faceIndicesOffset = offset;
        offset += faceIndices;
        const faceMaterialsOffset = offset;
        if (hasFaceTextures === 1) {
          offset += faceCount * 2;
        }
        const faceTextureIndicesOffset = offset;
        offset += textureIndices;
        const faceColorsOffset = offset;
        offset += faceCount * 2;
        const xVertexOffset = offset;
        offset += modelVerticesX;
        const yVertexOffset = offset;
        offset += modelVerticesY;
        const zVertexOffset = offset;
        offset += modelVerticesZ;
        const simpleTexturesOffset = offset;
        offset += simpleTextureFaceCount * 6;
        const complexTexturesOffset = offset;
        offset += complexTextureFaceCount * 6;
        let textureBytes = 6;
        if (this.version === 14) {
          textureBytes = 7;
        } else if (this.version >= 15) {
          textureBytes = 9;
        }
        const texturesScalesOffset = offset;
        offset += complexTextureFaceCount * textureBytes;
        const texturesRotationOffset = offset;
        offset += complexTextureFaceCount;
        const texturesDirectionOffset = offset;
        offset += complexTextureFaceCount;
        const texturesTranslationOffset = offset;
        offset += complexTextureFaceCount + cubeTextureFaceCount * 2;
        const particleEffectsOffset = offset;
        this.verticesCount = vertexCount;
        this.faceCount = faceCount;
        this.textureFaceCount = texFaceCount;
        this.verticesX = new Int32Array(vertexCount);
        this.verticesY = new Int32Array(vertexCount);
        this.verticesZ = new Int32Array(vertexCount);
        this.indices1 = new Int32Array(faceCount);
        this.indices2 = new Int32Array(faceCount);
        this.indices3 = new Int32Array(faceCount);
        if (hasVertexSkins === 1) {
          this.vertexSkins = new Int32Array(vertexCount);
        }
        if (hasFaceRenderTypes) {
          this.faceRenderTypes = new Int8Array(faceCount);
        }
        if (modelPriority === 255) {
          this.faceRenderPriorities = new Int8Array(faceCount);
        } else {
          this.priority = modelPriority;
        }
        if (hasFaceAlpha === 1) {
          this.faceAlphas = new Int8Array(faceCount);
        }
        if (hasFaceSkins === 1) {
          this.faceSkins = new Int32Array(faceCount);
        }
        if (hasFaceTextures === 1) {
          this.faceTextures = new Int16Array(faceCount);
        }
        if (hasFaceTextures === 1 && texFaceCount > 0) {
          this.textureCoords = new Int8Array(faceCount);
        }
        this.faceColors = new Uint16Array(faceCount);
        if (texFaceCount > 0) {
          this.textureMappingP = new Int16Array(texFaceCount);
          this.textureMappingM = new Int16Array(texFaceCount);
          this.textureMappingN = new Int16Array(texFaceCount);
          if (complexTextureFaceCount > 0) {
            this.textureScaleX = new Int32Array(complexTextureFaceCount);
            this.textureScaleY = new Int32Array(complexTextureFaceCount);
            this.textureScaleZ = new Int32Array(complexTextureFaceCount);
            this.textureRotation = new Int8Array(complexTextureFaceCount);
            this.textureDirection = new Int8Array(complexTextureFaceCount);
            this.textureSpeed = new Int32Array(complexTextureFaceCount);
          }
          if (cubeTextureFaceCount > 0) {
            this.textureTransU = new Int32Array(cubeTextureFaceCount);
            this.textureTransV = new Int32Array(cubeTextureFaceCount);
          }
        }
        buf1.offset = texFaceCount;
        buf2.offset = xVertexOffset;
        buf3.offset = yVertexOffset;
        buf4.offset = zVertexOffset;
        buf5.offset = vertexSkinsOffset;
        let lastVertX = 0;
        let lastVertY = 0;
        let lastVertZ = 0;
        for (let i = 0; i < vertexCount; i++) {
          const flag = buf1.readUnsignedByte();
          let deltaVertX = 0;
          if ((flag & 1) !== 0) {
            deltaVertX = buf2.readSmart2();
          }
          let deltaVertY = 0;
          if ((flag & 2) !== 0) {
            deltaVertY = buf3.readSmart2();
          }
          let deltaVertZ = 0;
          if ((flag & 4) !== 0) {
            deltaVertZ = buf4.readSmart2();
          }
          this.verticesX[i] = lastVertX + deltaVertX;
          this.verticesY[i] = lastVertY + deltaVertY;
          this.verticesZ[i] = lastVertZ + deltaVertZ;
          lastVertX = this.verticesX[i];
          lastVertY = this.verticesY[i];
          lastVertZ = this.verticesZ[i];
          if (hasVertexSkins === 1 && this.vertexSkins) {
            this.vertexSkins[i] = buf5.readUnsignedByte();
          }
        }
        buf1.offset = faceColorsOffset;
        buf2.offset = vertexFlagsOffset;
        buf3.offset = facePrioritiesOffset;
        buf4.offset = faceAlphasOffset;
        buf5.offset = faceSkinsOffset;
        buf6.offset = faceMaterialsOffset;
        buf7.offset = faceTextureIndicesOffset;
        for (let i = 0; i < faceCount; i++) {
          this.faceColors[i] = buf1.readUnsignedShort();
          if (hasFaceRenderTypes && this.faceRenderTypes) {
            this.faceRenderTypes[i] = buf2.readByte();
          }
          if (modelPriority === 255) {
            this.faceRenderPriorities[i] = buf3.readByte();
          }
          if (hasFaceAlpha === 1) {
            this.faceAlphas[i] = buf4.readByte();
          }
          if (hasFaceSkins === 1 && this.faceSkins) {
            this.faceSkins[i] = buf5.readUnsignedByte();
          }
          if (hasFaceTextures === 1 && this.faceTextures) {
            this.faceTextures[i] = buf6.readUnsignedShort() - 1;
          }
          if (this.textureCoords) {
            if (this.faceTextures && this.faceTextures[i] !== -1) {
              this.textureCoords[i] = buf7.readUnsignedByte() - 1;
            } else {
              this.textureCoords[i] = -1;
            }
          }
        }
        buf1.offset = faceIndicesOffset;
        buf2.offset = faceCompressTypeOffset;
        let index1 = 0;
        let index2 = 0;
        let index3 = 0;
        let var54 = 0;
        this.usedVertexCount = -1;
        for (let i = 0; i < faceCount; i++) {
          const type = buf2.readUnsignedByte();
          if (type === 1) {
            index1 = buf1.readSmart2() + var54;
            index2 = buf1.readSmart2() + index1;
            index3 = buf1.readSmart2() + index2;
            var54 = index3;
            this.indices1[i] = index1;
            this.indices2[i] = index2;
            this.indices3[i] = index3;
            if (index1 > this.usedVertexCount) {
              this.usedVertexCount = index1;
            }
            if (index2 > this.usedVertexCount) {
              this.usedVertexCount = index2;
            }
            if (index3 > this.usedVertexCount) {
              this.usedVertexCount = index3;
            }
          }
          if (type === 2) {
            index2 = index3;
            index3 = buf1.readSmart2() + var54;
            var54 = index3;
            this.indices1[i] = index1;
            this.indices2[i] = index2;
            this.indices3[i] = index3;
            if (index3 > this.usedVertexCount) {
              this.usedVertexCount = index3;
            }
          }
          if (type === 3) {
            index1 = index3;
            index3 = buf1.readSmart2() + var54;
            var54 = index3;
            this.indices1[i] = index1;
            this.indices2[i] = index2;
            this.indices3[i] = index3;
            if (index3 > this.usedVertexCount) {
              this.usedVertexCount = index3;
            }
          }
          if (type === 4) {
            const var57 = index1;
            index1 = index2;
            index2 = var57;
            index3 = buf1.readSmart2() + var54;
            var54 = index3;
            this.indices1[i] = index1;
            this.indices2[i] = var57;
            this.indices3[i] = index3;
            if (index3 > this.usedVertexCount) {
              this.usedVertexCount = index3;
            }
          }
        }
        this.usedVertexCount++;
        buf1.offset = simpleTexturesOffset;
        buf2.offset = complexTexturesOffset;
        buf3.offset = texturesScalesOffset;
        buf4.offset = texturesRotationOffset;
        buf5.offset = texturesDirectionOffset;
        buf6.offset = texturesTranslationOffset;
        this.decodeTextureMapping(buf1, buf2, buf3, buf4, buf5, buf6);
        buf1.offset = offset;
        if (this.version >= 13) {
          this.scaleDown(2);
        }
      }
      decodeTextureMapping(simpleBuffer, complexBuffer, scaleBuffer, rotationBuffer, directionBuffer, translationBuffer) {
        for (let i = 0; i < this.textureFaceCount; i++) {
          const type = this.textureRenderTypes[i] & 255;
          if (type === 0) {
            this.textureMappingP[i] = simpleBuffer.readUnsignedShort();
            this.textureMappingM[i] = simpleBuffer.readUnsignedShort();
            this.textureMappingN[i] = simpleBuffer.readUnsignedShort();
          }
          if (type === 1) {
            this.textureMappingP[i] = complexBuffer.readUnsignedShort();
            this.textureMappingM[i] = complexBuffer.readUnsignedShort();
            this.textureMappingN[i] = complexBuffer.readUnsignedShort();
            if (this.version < 15) {
              this.textureScaleX[i] = scaleBuffer.readUnsignedShort();
              if (this.version >= 14) {
                this.textureScaleY[i] = scaleBuffer.readMedium();
              } else {
                this.textureScaleY[i] = scaleBuffer.readUnsignedShort();
              }
              this.textureScaleZ[i] = scaleBuffer.readUnsignedShort();
            } else {
              this.textureScaleX[i] = scaleBuffer.readMedium();
              this.textureScaleY[i] = scaleBuffer.readMedium();
              this.textureScaleZ[i] = scaleBuffer.readMedium();
            }
            this.textureRotation[i] = rotationBuffer.readByte();
            this.textureDirection[i] = directionBuffer.readByte();
            this.textureSpeed[i] = translationBuffer.readByte();
          }
          if (type === 2) {
            this.textureMappingP[i] = complexBuffer.readUnsignedShort();
            this.textureMappingM[i] = complexBuffer.readUnsignedShort();
            this.textureMappingN[i] = complexBuffer.readUnsignedShort();
            if (this.version < 15) {
              this.textureScaleX[i] = scaleBuffer.readUnsignedShort();
              if (this.version >= 14) {
                this.textureScaleY[i] = scaleBuffer.readMedium();
              } else {
                this.textureScaleY[i] = scaleBuffer.readUnsignedShort();
              }
              this.textureScaleZ[i] = scaleBuffer.readUnsignedShort();
            } else {
              this.textureScaleX[i] = scaleBuffer.readMedium();
              this.textureScaleY[i] = scaleBuffer.readMedium();
              this.textureScaleZ[i] = scaleBuffer.readMedium();
            }
            this.textureRotation[i] = rotationBuffer.readByte();
            this.textureDirection[i] = directionBuffer.readByte();
            this.textureSpeed[i] = translationBuffer.readByte();
            this.textureTransU[i] = translationBuffer.readByte();
            this.textureTransV[i] = translationBuffer.readByte();
          }
          if (type === 3) {
            this.textureMappingP[i] = complexBuffer.readUnsignedShort();
            this.textureMappingM[i] = complexBuffer.readUnsignedShort();
            this.textureMappingN[i] = complexBuffer.readUnsignedShort();
            if (this.version < 15) {
              this.textureScaleX[i] = scaleBuffer.readUnsignedShort();
              if (this.version >= 14) {
                this.textureScaleY[i] = scaleBuffer.readMedium();
              } else {
                this.textureScaleY[i] = scaleBuffer.readUnsignedShort();
              }
              this.textureScaleZ[i] = scaleBuffer.readUnsignedShort();
            } else {
              this.textureScaleX[i] = scaleBuffer.readMedium();
              this.textureScaleY[i] = scaleBuffer.readMedium();
              this.textureScaleZ[i] = scaleBuffer.readMedium();
            }
            this.textureRotation[i] = rotationBuffer.readByte();
            this.textureDirection[i] = directionBuffer.readByte();
            this.textureSpeed[i] = translationBuffer.readByte();
          }
        }
      }
      decodeOld(data) {
        this.version = 0;
        let hasRenderType = false;
        let isTextured = false;
        const buf1 = new ByteBuffer(data);
        const buf2 = new ByteBuffer(data);
        const buf3 = new ByteBuffer(data);
        const buf4 = new ByteBuffer(data);
        const buf5 = new ByteBuffer(data);
        buf1.offset = data.length - 18;
        const vertexCount = buf1.readUnsignedShort();
        const faceCount = buf1.readUnsignedShort();
        const texTriangleCount = buf1.readUnsignedByte();
        const usesTextures = buf1.readUnsignedByte();
        const var13 = buf1.readUnsignedByte();
        const var14 = buf1.readUnsignedByte();
        const var15 = buf1.readUnsignedByte();
        const var16 = buf1.readUnsignedByte();
        const var17 = buf1.readUnsignedShort();
        const var18 = buf1.readUnsignedShort();
        const var19 = buf1.readUnsignedShort();
        const var20 = buf1.readUnsignedShort();
        let var21 = 0;
        let var45 = var21 + vertexCount;
        let var23 = var45;
        var45 += faceCount;
        const var24 = var45;
        if (var13 === 255) {
          var45 += faceCount;
        }
        const var25 = var45;
        if (var15 === 1) {
          var45 += faceCount;
        }
        const var26 = var45;
        if (usesTextures === 1) {
          var45 += faceCount;
        }
        const var27 = var45;
        if (var16 === 1) {
          var45 += vertexCount;
        }
        const var28 = var45;
        if (var14 === 1) {
          var45 += faceCount;
        }
        const var29 = var45;
        var45 += var20;
        const var30 = var45;
        var45 += faceCount * 2;
        const var31 = var45;
        var45 += texTriangleCount * 6;
        const var32 = var45;
        var45 += var17;
        const var33 = var45;
        var45 += var18;
        this.verticesCount = vertexCount;
        this.faceCount = faceCount;
        this.textureFaceCount = texTriangleCount;
        this.verticesX = new Int32Array(vertexCount);
        this.verticesY = new Int32Array(vertexCount);
        this.verticesZ = new Int32Array(vertexCount);
        this.indices1 = new Int32Array(faceCount);
        this.indices2 = new Int32Array(faceCount);
        this.indices3 = new Int32Array(faceCount);
        if (texTriangleCount > 0) {
          this.textureRenderTypes = new Int8Array(texTriangleCount);
          this.textureMappingP = new Int16Array(texTriangleCount);
          this.textureMappingM = new Int16Array(texTriangleCount);
          this.textureMappingN = new Int16Array(texTriangleCount);
        }
        if (var16 === 1) {
          this.vertexSkins = new Int32Array(vertexCount);
        }
        if (usesTextures === 1) {
          this.faceRenderTypes = new Int8Array(faceCount);
          this.textureCoords = new Int8Array(faceCount);
          this.faceTextures = new Int16Array(faceCount);
        }
        if (var13 === 255) {
          this.faceRenderPriorities = new Int8Array(faceCount);
        } else {
          this.priority = var13;
        }
        if (var14 === 1) {
          this.faceAlphas = new Int8Array(faceCount);
        }
        if (var15 === 1) {
          this.faceSkins = new Int32Array(faceCount);
        }
        this.faceColors = new Uint16Array(faceCount);
        buf1.offset = var21;
        buf2.offset = var32;
        buf3.offset = var33;
        buf4.offset = var45;
        buf5.offset = var27;
        let lastVertX = 0;
        let lastVertY = 0;
        let lastVertZ = 0;
        for (let i = 0; i < vertexCount; i++) {
          const flag = buf1.readUnsignedByte();
          let deltaVertX = 0;
          if ((flag & 1) !== 0) {
            deltaVertX = buf2.readSmart2();
          }
          let deltaVertY = 0;
          if ((flag & 2) !== 0) {
            deltaVertY = buf3.readSmart2();
          }
          let deltaVertZ = 0;
          if ((flag & 4) !== 0) {
            deltaVertZ = buf4.readSmart2();
          }
          this.verticesX[i] = lastVertX + deltaVertX;
          this.verticesY[i] = lastVertY + deltaVertY;
          this.verticesZ[i] = lastVertZ + deltaVertZ;
          lastVertX = this.verticesX[i];
          lastVertY = this.verticesY[i];
          lastVertZ = this.verticesZ[i];
          if (var16 === 1 && this.vertexSkins) {
            this.vertexSkins[i] = buf5.readUnsignedByte();
          }
        }
        buf1.offset = var30;
        buf2.offset = var26;
        buf3.offset = var24;
        buf4.offset = var28;
        buf5.offset = var25;
        for (let i = 0; i < faceCount; i++) {
          this.faceColors[i] = buf1.readUnsignedShort();
          if (usesTextures === 1 && this.faceRenderTypes && this.textureCoords && this.faceTextures) {
            const flag = buf2.readUnsignedByte();
            if ((flag & 1) === 1) {
              this.faceRenderTypes[i] = 1;
              hasRenderType = true;
            } else {
              this.faceRenderTypes[i] = 0;
            }
            if ((flag & 2) === 2) {
              this.textureCoords[i] = flag >> 2;
              this.faceTextures[i] = this.faceColors[i];
              this.faceColors[i] = 127;
              if (this.faceTextures[i] !== -1) {
                isTextured = true;
              }
            } else {
              this.textureCoords[i] = -1;
              this.faceTextures[i] = -1;
            }
          }
          if (var13 === 255) {
            this.faceRenderPriorities[i] = buf3.readByte();
          }
          if (var14 === 1) {
            this.faceAlphas[i] = buf4.readByte();
          }
          if (var15 === 1 && this.faceSkins) {
            this.faceSkins[i] = buf5.readUnsignedByte();
          }
        }
        buf1.offset = var29;
        buf2.offset = var23;
        let index1 = 0;
        let index2 = 0;
        let index3 = 0;
        let lastIndex = 0;
        this.usedVertexCount = -1;
        for (let i = 0; i < faceCount; i++) {
          const type = buf2.readUnsignedByte();
          if (type === 1) {
            index1 = buf1.readSmart2() + lastIndex;
            index2 = buf1.readSmart2() + index1;
            index3 = buf1.readSmart2() + index2;
            lastIndex = index3;
            this.indices1[i] = index1;
            this.indices2[i] = index2;
            this.indices3[i] = index3;
            if (index1 > this.usedVertexCount) {
              this.usedVertexCount = index1;
            }
            if (index2 > this.usedVertexCount) {
              this.usedVertexCount = index2;
            }
            if (index3 > this.usedVertexCount) {
              this.usedVertexCount = index3;
            }
          }
          if (type === 2) {
            index2 = index3;
            index3 = buf1.readSmart2() + lastIndex;
            lastIndex = index3;
            this.indices1[i] = index1;
            this.indices2[i] = index2;
            this.indices3[i] = index3;
            if (index3 > this.usedVertexCount) {
              this.usedVertexCount = index3;
            }
          }
          if (type === 3) {
            index1 = index3;
            index3 = buf1.readSmart2() + lastIndex;
            lastIndex = index3;
            this.indices1[i] = index1;
            this.indices2[i] = index2;
            this.indices3[i] = index3;
            if (index3 > this.usedVertexCount) {
              this.usedVertexCount = index3;
            }
          }
          if (type === 4) {
            const var44 = index1;
            index1 = index2;
            index2 = var44;
            index3 = buf1.readSmart2() + lastIndex;
            lastIndex = index3;
            this.indices1[i] = index1;
            this.indices2[i] = var44;
            this.indices3[i] = index3;
            if (index3 > this.usedVertexCount) {
              this.usedVertexCount = index3;
            }
          }
        }
        this.usedVertexCount++;
        buf1.offset = var31;
        for (let i = 0; i < texTriangleCount; i++) {
          this.textureRenderTypes[i] = 0;
          this.textureMappingP[i] = buf1.readUnsignedShort();
          this.textureMappingM[i] = buf1.readUnsignedShort();
          this.textureMappingN[i] = buf1.readUnsignedShort();
        }
        if (this.textureCoords) {
          let hasValidTexFace = false;
          for (let i = 0; i < faceCount; i++) {
            const index = this.textureCoords[i] & 255;
            if (index !== 255) {
              if (this.indices1[i] === (this.textureMappingP[index] & 65535) && this.indices2[i] === (this.textureMappingM[index] & 65535) && this.indices3[i] === (this.textureMappingN[index] & 65535)) {
                this.textureCoords[i] = -1;
              } else {
                hasValidTexFace = true;
              }
            }
          }
          if (!hasValidTexFace) {
            this.textureCoords = void 0;
          }
        }
        if (!isTextured) {
          this.faceTextures = void 0;
        }
        if (!hasRenderType) {
          this.faceRenderTypes = void 0;
        }
      }
      decodeLegacy(loader, meta) {
        let hasRenderType = false;
        let isTextured = false;
        this.verticesCount = meta.vertexCount;
        this.faceCount = meta.triangleCount;
        this.textureFaceCount = meta.texturedTriangleCount;
        this.verticesX = new Int32Array(this.verticesCount);
        this.verticesY = new Int32Array(this.verticesCount);
        this.verticesZ = new Int32Array(this.verticesCount);
        this.indices1 = new Int32Array(this.faceCount);
        this.indices2 = new Int32Array(this.faceCount);
        this.indices3 = new Int32Array(this.faceCount);
        if (this.textureFaceCount > 0) {
          this.textureRenderTypes = new Int8Array(this.textureFaceCount);
          this.textureMappingP = new Int16Array(this.textureFaceCount);
          this.textureMappingM = new Int16Array(this.textureFaceCount);
          this.textureMappingN = new Int16Array(this.textureFaceCount);
        }
        if (meta.vertexLabelsOffset >= 0) {
          this.vertexSkins = new Int32Array(this.verticesCount);
        }
        if (meta.faceInfosOffset >= 0) {
          this.faceRenderTypes = new Int8Array(this.faceCount);
          this.textureCoords = new Int8Array(this.faceCount);
          this.faceTextures = new Int16Array(this.faceCount);
        }
        if (meta.facePrioritiesOffset >= 0) {
          this.faceRenderPriorities = new Int8Array(this.faceCount);
        } else {
          this.priority = -meta.facePrioritiesOffset - 1;
        }
        if (meta.faceAlphasOffset >= 0) {
          this.faceAlphas = new Int8Array(this.faceCount);
        }
        if (meta.faceLabelsOffset >= 0) {
          this.faceSkins = new Int32Array(this.faceCount);
        }
        this.faceColors = new Uint16Array(this.faceCount);
        loader.point1.offset = meta.vertexFlagsOffset;
        loader.point2.offset = meta.vertexXOffset;
        loader.point3.offset = meta.vertexYOffset;
        loader.point4.offset = meta.vertexZOffset;
        loader.point5.offset = meta.vertexLabelsOffset;
        let lastVertX = 0;
        let lastVertY = 0;
        let lastVertZ = 0;
        for (let i = 0; i < this.verticesCount; i++) {
          const flag = loader.point1.readUnsignedByte();
          let deltaVertX = 0;
          if ((flag & 1) !== 0) {
            deltaVertX = loader.point2.readSmart2();
          }
          let deltaVertY = 0;
          if ((flag & 2) !== 0) {
            deltaVertY = loader.point3.readSmart2();
          }
          let deltaVertZ = 0;
          if ((flag & 4) !== 0) {
            deltaVertZ = loader.point4.readSmart2();
          }
          this.verticesX[i] = lastVertX + deltaVertX;
          this.verticesY[i] = lastVertY + deltaVertY;
          this.verticesZ[i] = lastVertZ + deltaVertZ;
          lastVertX = this.verticesX[i];
          lastVertY = this.verticesY[i];
          lastVertZ = this.verticesZ[i];
          if (this.vertexSkins) {
            this.vertexSkins[i] = loader.point5.readUnsignedByte();
          }
        }
        loader.face1.offset = meta.faceColorsOffset;
        loader.face2.offset = meta.faceInfosOffset;
        loader.face3.offset = meta.facePrioritiesOffset;
        loader.face4.offset = meta.faceAlphasOffset;
        loader.face5.offset = meta.faceLabelsOffset;
        for (let i = 0; i < this.faceCount; i++) {
          this.faceColors[i] = loader.face1.readUnsignedShort();
          if (this.faceRenderTypes && this.textureCoords && this.faceTextures) {
            const flag = loader.face2.readUnsignedByte();
            if ((flag & 1) === 1) {
              this.faceRenderTypes[i] = 1;
              hasRenderType = true;
            } else {
              this.faceRenderTypes[i] = 0;
            }
            if ((flag & 2) === 2) {
              this.textureCoords[i] = flag >> 2;
              this.faceTextures[i] = this.faceColors[i];
              this.faceColors[i] = 127;
              if (this.faceTextures[i] !== -1) {
                isTextured = true;
              }
            } else {
              this.textureCoords[i] = -1;
              this.faceTextures[i] = -1;
            }
          }
          if (this.faceRenderPriorities) {
            this.faceRenderPriorities[i] = loader.face3.readByte();
          }
          if (this.faceAlphas) {
            this.faceAlphas[i] = loader.face4.readByte();
          }
          if (this.faceSkins) {
            this.faceSkins[i] = loader.face5.readUnsignedByte();
          }
        }
        loader.vertex1.offset = meta.faceVerticesOffset;
        loader.vertex2.offset = meta.faceOrientationsOffset;
        let index1 = 0;
        let index2 = 0;
        let index3 = 0;
        let lastIndex = 0;
        this.usedVertexCount = -1;
        for (let i = 0; i < this.faceCount; i++) {
          const type = loader.vertex2.readUnsignedByte();
          if (type === 1) {
            index1 = loader.vertex1.readSmart2() + lastIndex;
            index2 = loader.vertex1.readSmart2() + index1;
            index3 = loader.vertex1.readSmart2() + index2;
            lastIndex = index3;
            this.indices1[i] = index1;
            this.indices2[i] = index2;
            this.indices3[i] = index3;
            if (index1 > this.usedVertexCount) {
              this.usedVertexCount = index1;
            }
            if (index2 > this.usedVertexCount) {
              this.usedVertexCount = index2;
            }
            if (index3 > this.usedVertexCount) {
              this.usedVertexCount = index3;
            }
          }
          if (type === 2) {
            index2 = index3;
            index3 = loader.vertex1.readSmart2() + lastIndex;
            lastIndex = index3;
            this.indices1[i] = index1;
            this.indices2[i] = index2;
            this.indices3[i] = index3;
            if (index3 > this.usedVertexCount) {
              this.usedVertexCount = index3;
            }
          }
          if (type === 3) {
            index1 = index3;
            index3 = loader.vertex1.readSmart2() + lastIndex;
            lastIndex = index3;
            this.indices1[i] = index1;
            this.indices2[i] = index2;
            this.indices3[i] = index3;
            if (index3 > this.usedVertexCount) {
              this.usedVertexCount = index3;
            }
          }
          if (type === 4) {
            const temp = index1;
            index1 = index2;
            index2 = temp;
            index3 = loader.vertex1.readSmart2() + lastIndex;
            lastIndex = index3;
            this.indices1[i] = index1;
            this.indices2[i] = temp;
            this.indices3[i] = index3;
            if (index3 > this.usedVertexCount) {
              this.usedVertexCount = index3;
            }
          }
        }
        this.usedVertexCount++;
        loader.axis.offset = meta.faceTextureAxisOffset * 6;
        for (let i = 0; i < this.textureFaceCount; i++) {
          this.textureRenderTypes[i] = 0;
          this.textureMappingP[i] = loader.axis.readUnsignedShort();
          this.textureMappingM[i] = loader.axis.readUnsignedShort();
          this.textureMappingN[i] = loader.axis.readUnsignedShort();
        }
        if (this.textureCoords) {
          let hasValidTexFace = false;
          for (let i = 0; i < this.faceCount; i++) {
            const index = this.textureCoords[i] & 255;
            if (index !== 255) {
              if (this.indices1[i] === (this.textureMappingP[index] & 65535) && this.indices2[i] === (this.textureMappingM[index] & 65535) && this.indices3[i] === (this.textureMappingN[index] & 65535)) {
                this.textureCoords[i] = -1;
              } else {
                hasValidTexFace = true;
              }
            }
          }
          if (!hasValidTexFace) {
            this.textureCoords = void 0;
          }
        }
        if (!isTextured) {
          this.faceTextures = void 0;
        }
        if (!hasRenderType) {
          this.faceRenderTypes = void 0;
        }
      }
      copyFrom(model, shallowIndices, shallowVertices, shallowColors, shallowTextures) {
        this.verticesCount = model.verticesCount;
        this.usedVertexCount = model.usedVertexCount;
        this.faceCount = model.faceCount;
        this.textureFaceCount = model.textureFaceCount;
        if (shallowIndices) {
          this.indices1 = model.indices1;
          this.indices2 = model.indices2;
          this.indices3 = model.indices3;
        } else {
          this.indices1 = new Int32Array(this.faceCount);
          this.indices2 = new Int32Array(this.faceCount);
          this.indices3 = new Int32Array(this.faceCount);
          for (let i = 0; i < this.faceCount; i++) {
            this.indices1[i] = model.indices1[i];
            this.indices2[i] = model.indices2[i];
            this.indices3[i] = model.indices3[i];
          }
        }
        if (shallowVertices) {
          this.verticesX = model.verticesX;
          this.verticesY = model.verticesY;
          this.verticesZ = model.verticesZ;
        } else {
          this.verticesX = new Int32Array(this.verticesCount);
          this.verticesY = new Int32Array(this.verticesCount);
          this.verticesZ = new Int32Array(this.verticesCount);
          for (let i = 0; i < this.verticesCount; i++) {
            this.verticesX[i] = model.verticesX[i];
            this.verticesY[i] = model.verticesY[i];
            this.verticesZ[i] = model.verticesZ[i];
          }
        }
        if (shallowColors) {
          this.faceColors = model.faceColors;
        } else {
          this.faceColors = new Uint16Array(this.faceCount);
          for (let i = 0; i < this.faceCount; i++) {
            this.faceColors[i] = model.faceColors[i];
          }
        }
        if (!shallowTextures && model.faceTextures) {
          this.faceTextures = new Int16Array(this.faceCount);
          for (let i = 0; i < this.faceCount; i++) {
            this.faceTextures[i] = model.faceTextures[i];
          }
        } else {
          this.faceTextures = model.faceTextures;
        }
        this.faceAlphas = model.faceAlphas;
        this.faceRenderTypes = model.faceRenderTypes;
        this.faceRenderPriorities = model.faceRenderPriorities;
        this.textureCoords = model.textureCoords;
        this.priority = model.priority;
        this.textureRenderTypes = model.textureRenderTypes;
        this.textureMappingP = model.textureMappingP;
        this.textureMappingM = model.textureMappingM;
        this.textureMappingN = model.textureMappingN;
        this.textureScaleX = model.textureScaleX;
        this.textureScaleY = model.textureScaleY;
        this.textureScaleZ = model.textureScaleZ;
        this.textureRotation = model.textureRotation;
        this.textureDirection = model.textureDirection;
        this.textureSpeed = model.textureSpeed;
        this.textureTransU = model.textureTransU;
        this.textureTransV = model.textureTransV;
        this.vertexSkins = model.vertexSkins;
        this.faceSkins = model.faceSkins;
        this.vertexLabels = model.vertexLabels;
        this.faceLabels = model.faceLabels;
        this.normals = model.normals;
        this.faceNormals = model.faceNormals;
        this.mergedNormals = model.mergedNormals;
        this.animMayaGroups = model.animMayaGroups;
        this.animMayaScales = model.animMayaScales;
        this.ambient = model.ambient;
        this.contrast = model.contrast;
      }
      copy() {
        const model = new _ModelData();
        if (this.faceRenderTypes) {
          model.faceRenderTypes = new Int8Array(this.faceCount);
          for (let i = 0; i < this.faceCount; i++) {
            model.faceRenderTypes[i] = this.faceRenderTypes[i];
          }
        }
        model.verticesCount = this.verticesCount;
        model.usedVertexCount = this.usedVertexCount;
        model.faceCount = this.faceCount;
        model.textureFaceCount = this.textureFaceCount;
        model.verticesX = this.verticesX;
        model.verticesY = this.verticesY;
        model.verticesZ = this.verticesZ;
        model.indices1 = this.indices1;
        model.indices2 = this.indices2;
        model.indices3 = this.indices3;
        model.faceRenderPriorities = this.faceRenderPriorities;
        model.faceAlphas = this.faceAlphas;
        model.textureCoords = this.textureCoords;
        model.faceColors = this.faceColors;
        model.faceTextures = this.faceTextures;
        model.priority = this.priority;
        model.textureRenderTypes = this.textureRenderTypes;
        model.textureMappingP = this.textureMappingP;
        model.textureMappingM = this.textureMappingM;
        model.textureMappingN = this.textureMappingN;
        model.textureScaleX = this.textureScaleX;
        model.textureScaleY = this.textureScaleY;
        model.textureScaleZ = this.textureScaleZ;
        model.textureRotation = this.textureRotation;
        model.textureDirection = this.textureDirection;
        model.textureSpeed = this.textureSpeed;
        model.textureTransU = this.textureTransU;
        model.textureTransV = this.textureTransV;
        model.vertexSkins = this.vertexSkins;
        model.faceSkins = this.faceSkins;
        model.vertexLabels = this.vertexLabels;
        model.faceLabels = this.faceLabels;
        model.normals = this.normals;
        model.faceNormals = this.faceNormals;
        model.ambient = this.ambient;
        model.contrast = this.contrast;
        return model;
      }
      contourGround(type, param, heightMap, heightMapAbove, sceneX, sceneHeight, sceneZ) {
        if (this.usedVertexCount === 0) {
          return this;
        }
        this.calculateBounds();
        let startX = sceneX + this.minX;
        let endX = sceneX + this.maxX;
        let startY = sceneZ + this.minZ;
        let endY = sceneZ + this.maxZ;
        if ((type === 1 || type === 2 || type === 3 || type === 5) && (startX < 0 || endX + 128 >> 7 >= heightMap.length || startY < 0 || endY + 128 >> 7 >= heightMap[0].length)) {
          return this;
        }
        if (type === 4 || type === 5) {
          if (heightMapAbove === void 0) {
            return this;
          }
          if (startX < 0 || endX + 128 >> 7 >= heightMapAbove.length || startY < 0 || endY + 128 >> 7 >= heightMapAbove[0].length) {
            return this;
          }
        } else {
          startX >>= 7;
          endX = endX + 127 >> 7;
          startY >>= 7;
          endY = endY + 127 >> 7;
          if (heightMap[startX][startY] === sceneHeight && heightMap[endX][startY] === sceneHeight && heightMap[startX][endY] === sceneHeight && heightMap[endX][endY] === sceneHeight) {
            return this;
          }
        }
        const model = new _ModelData();
        model.verticesCount = this.verticesCount;
        model.usedVertexCount = this.usedVertexCount;
        model.faceCount = this.faceCount;
        model.textureFaceCount = this.textureFaceCount;
        model.verticesX = this.verticesX;
        model.verticesZ = this.verticesZ;
        model.indices1 = this.indices1;
        model.indices2 = this.indices2;
        model.indices3 = this.indices3;
        model.faceRenderTypes = this.faceRenderTypes;
        model.faceRenderPriorities = this.faceRenderPriorities;
        model.faceAlphas = this.faceAlphas;
        model.textureCoords = this.textureCoords;
        model.faceColors = this.faceColors;
        model.faceTextures = this.faceTextures;
        model.priority = this.priority;
        model.textureRenderTypes = this.textureRenderTypes;
        model.textureMappingP = this.textureMappingP;
        model.textureMappingM = this.textureMappingM;
        model.textureMappingN = this.textureMappingN;
        model.textureScaleX = this.textureScaleX;
        model.textureScaleY = this.textureScaleY;
        model.textureScaleZ = this.textureScaleZ;
        model.textureRotation = this.textureRotation;
        model.textureDirection = this.textureDirection;
        model.textureSpeed = this.textureSpeed;
        model.textureTransU = this.textureTransU;
        model.textureTransV = this.textureTransV;
        model.vertexSkins = this.vertexSkins;
        model.faceSkins = this.faceSkins;
        model.vertexLabels = this.vertexLabels;
        model.faceLabels = this.faceLabels;
        model.ambient = this.ambient;
        model.contrast = this.contrast;
        model.verticesY = this.verticesY;
        model.contourVerticesY = new Int32Array(model.verticesCount);
        if (type === 1) {
          for (let i = 0; i < model.usedVertexCount; i++) {
            const vx = this.verticesX[i] + sceneX;
            const vz = this.verticesZ[i] + sceneZ;
            const rx = vx & 127;
            const rz = vz & 127;
            const tx = vx >> 7;
            const tz = vz >> 7;
            const h0 = heightMap[tx][tz] * (128 - rx) + heightMap[tx + 1][tz] * rx >> 7;
            const h1 = heightMap[tx][tz + 1] * (128 - rx) + heightMap[tx + 1][tz + 1] * rx >> 7;
            const height = h0 * (128 - rz) + h1 * rz >> 7;
            model.contourVerticesY[i] = this.verticesY[i] + height - sceneHeight;
          }
          for (let i = model.usedVertexCount; i < model.verticesCount; i++) {
            const vx = this.verticesX[i] + sceneX;
            const vz = this.verticesZ[i] + sceneZ;
            const rx = vx & 127;
            const rz = vz & 127;
            const tx = vx >> 7;
            const tz = vz >> 7;
            if (tx >= 0 && tx < heightMap.length - 1 && tz >= 0 && tz < heightMap[0].length - 1) {
              const h0 = heightMap[tx][tz] * (128 - rx) + heightMap[tx + 1][tz] * rx >> 7;
              const h1 = heightMap[tx][tz + 1] * (128 - rx) + heightMap[tx + 1][tz + 1] * rx >> 7;
              const height = h0 * (128 - rz) + h1 * rz >> 7;
              model.contourVerticesY[i] = this.verticesY[i] + height - sceneHeight;
            }
          }
        } else if (type === 2) {
          for (let i = 0; i < model.usedVertexCount; i++) {
            const yRatio = (this.verticesY[i] << 16) / -this.height | 0;
            if (yRatio < param) {
              const vx = this.verticesX[i] + sceneX;
              const vz = this.verticesZ[i] + sceneZ;
              const rx = vx & 127;
              const rz = vz & 127;
              const tx = vx >> 7;
              const tz = vz >> 7;
              const h0 = heightMap[tx][tz] * (128 - rx) + heightMap[tx + 1][tz] * rx >> 7;
              const h1 = heightMap[tx][tz + 1] * (128 - rx) + heightMap[tx + 1][tz + 1] * rx >> 7;
              const height = h0 * (128 - rz) + h1 * rz >> 7;
              model.contourVerticesY[i] = this.verticesY[i] + (height - sceneHeight) * (param - yRatio) / param;
            } else {
              model.contourVerticesY[i] = this.verticesY[i];
            }
          }
          for (let i = model.usedVertexCount; i < model.verticesCount; i++) {
            const yRatio = (this.verticesY[i] << 16) / -this.height | 0;
            if (yRatio < param) {
              const vx = this.verticesX[i] + sceneX;
              const vz = this.verticesZ[i] + sceneZ;
              const rx = vx & 127;
              const rz = vz & 127;
              const tx = vx >> 7;
              const tz = vz >> 7;
              if (tx >= 0 && tx < heightMap.length - 1 && tz >= 0 && tz < heightMap[0].length - 1) {
                const h0 = heightMap[tx][tz] * (128 - rx) + heightMap[tx + 1][tz] * rx >> 7;
                const h1 = heightMap[tx][tz + 1] * (128 - rx) + heightMap[tx + 1][tz + 1] * rx >> 7;
                const height = h0 * (128 - rz) + h1 * rz >> 7;
                model.contourVerticesY[i] = this.verticesY[i] + (height - sceneHeight) * (param - yRatio) / param;
              }
            } else {
              model.contourVerticesY[i] = this.verticesY[i];
            }
          }
        } else if (type === 3) {
          const clampLimit = param !== 0 ? Math.abs(param) : void 0;
          for (let i = 0; i < model.usedVertexCount; i++) {
            const vx = this.verticesX[i] + sceneX;
            const vz = this.verticesZ[i] + sceneZ;
            const rx = vx & 127;
            const rz = vz & 127;
            const tx = vx >> 7;
            const tz = vz >> 7;
            if (tx >= 0 && tx < heightMap.length - 1 && tz >= 0 && tz < heightMap[0].length - 1) {
              const h0 = heightMap[tx][tz] * (128 - rx) + heightMap[tx + 1][tz] * rx >> 7;
              const h1 = heightMap[tx][tz + 1] * (128 - rx) + heightMap[tx + 1][tz + 1] * rx >> 7;
              let delta = (h0 * (128 - rz) + h1 * rz >> 7) - sceneHeight;
              if (clampLimit !== void 0 && clampLimit > 0) {
                if (delta > clampLimit) {
                  delta = clampLimit;
                } else if (delta < -clampLimit) {
                  delta = -clampLimit;
                }
              }
              model.contourVerticesY[i] = this.verticesY[i] + delta;
            } else {
              model.contourVerticesY[i] = this.verticesY[i];
            }
          }
        } else if (type === 4) {
          const deltaY = this.maxY - this.minY;
          for (let i = 0; i < model.usedVertexCount; i++) {
            const vx = this.verticesX[i] + sceneX;
            const vz = this.verticesZ[i] + sceneZ;
            const rx = vx & 127;
            const rz = vz & 127;
            const tx = vx >> 7;
            const tz = vz >> 7;
            const h0 = heightMapAbove[tx][tz] * (128 - rx) + heightMapAbove[tx + 1][tz] * rx >> 7;
            const h1 = heightMapAbove[tx][tz + 1] * (128 - rx) + heightMapAbove[tx + 1][tz + 1] * rx >> 7;
            const height = h0 * (128 - rz) + h1 * rz >> 7;
            model.contourVerticesY[i] = this.verticesY[i] + height - sceneHeight + deltaY;
          }
        } else if (type === 5) {
          const deltaY = this.maxY - this.minY;
          for (let i = 0; i < model.usedVertexCount; i++) {
            const vx = this.verticesX[i] + sceneX;
            const vz = this.verticesZ[i] + sceneZ;
            const rx = vx & 127;
            const rz = vz & 127;
            const tx = vx >> 7;
            const tz = vz >> 7;
            let h0 = heightMap[tx][tz] * (128 - rx) + heightMap[tx + 1][tz] * rx >> 7;
            let h1 = heightMap[tx][tz + 1] * (128 - rx) + heightMap[tx + 1][tz + 1] * rx >> 7;
            const height = h0 * (128 - rz) + h1 * rz >> 7;
            h0 = heightMapAbove[tx][tz] * (128 - rx) + heightMapAbove[tx + 1][tz] * rx >> 7;
            h1 = heightMapAbove[tx][tz + 1] * (128 - rx) + heightMapAbove[tx + 1][tz + 1] * rx >> 7;
            const heightAbove = h0 * (128 - rz) + h1 * rz >> 7;
            const deltaHeight = height - heightAbove;
            model.contourVerticesY[i] = (((this.verticesY[i] << 8) / deltaY | 0) * deltaHeight >> 8) - (sceneHeight - height);
          }
        }
        model.invalidate();
        return model;
      }
      computeAnimationTables() {
        let skin;
        if (this.vertexSkins) {
          const labelCounts = new Array(256).fill(0);
          let highestSkin = 0;
          const vertexCount = this.usedVertexCount;
          for (let i = 0; i < vertexCount; i++) {
            skin = this.vertexSkins[i];
            if (skin >= 0) {
              labelCounts[skin]++;
              if (skin > highestSkin) {
                highestSkin = skin;
              }
            }
          }
          this.vertexLabels = new Array(highestSkin + 1);
          for (let i = 0; i <= highestSkin; i++) {
            this.vertexLabels[i] = new Int32Array(labelCounts[i]);
            labelCounts[i] = 0;
          }
          for (let label = 0; label < vertexCount; label++) {
            const skin2 = this.vertexSkins[label];
            if (skin2 >= 0) {
              this.vertexLabels[skin2][labelCounts[skin2]++] = label;
            }
          }
          this.vertexSkins = void 0;
        }
        if (this.faceSkins) {
          const labelCounts = new Array(256).fill(0);
          let highestSkin = 0;
          for (let i = 0; i < this.faceCount; i++) {
            skin = this.faceSkins[i];
            if (skin >= 0) {
              labelCounts[skin]++;
              if (skin > highestSkin) {
                highestSkin = skin;
              }
            }
          }
          this.faceLabels = new Array(highestSkin + 1);
          for (let i = 0; i <= highestSkin; i++) {
            this.faceLabels[i] = new Int32Array(labelCounts[i]);
            labelCounts[i] = 0;
          }
          for (let label = 0; label < this.faceCount; label++) {
            const skin2 = this.faceSkins[label];
            if (skin2 >= 0) {
              this.faceLabels[skin2][labelCounts[skin2]++] = label;
            }
          }
          this.faceSkins = void 0;
        }
      }
      rotate90() {
        for (let i = 0; i < this.verticesCount; i++) {
          const temp = this.verticesX[i];
          this.verticesX[i] = this.verticesZ[i];
          this.verticesZ[i] = -temp;
        }
        this.invalidate();
      }
      rotate180() {
        for (let i = 0; i < this.verticesCount; i++) {
          this.verticesX[i] = -this.verticesX[i];
          this.verticesZ[i] = -this.verticesZ[i];
        }
        this.invalidate();
      }
      rotate270() {
        for (let i = 0; i < this.verticesCount; i++) {
          const temp = this.verticesZ[i];
          this.verticesZ[i] = this.verticesX[i];
          this.verticesX[i] = -temp;
        }
        this.invalidate();
      }
      rotate(angle) {
        const sin = SINE[angle];
        const cos = COSINE[angle];
        for (let i = 0; i < this.verticesCount; i++) {
          const temp = sin * this.verticesZ[i] + cos * this.verticesX[i] >> 16;
          this.verticesZ[i] = cos * this.verticesZ[i] - sin * this.verticesX[i] >> 16;
          this.verticesX[i] = temp;
        }
        this.invalidate();
      }
      translate(x, y, z) {
        for (let i = 0; i < this.verticesCount; i++) {
          this.verticesX[i] += x;
          this.verticesY[i] += y;
          this.verticesZ[i] += z;
        }
        this.invalidate();
      }
      recolor(from, to) {
        for (let i = 0; i < this.faceCount; i++) {
          if (this.faceColors[i] === from) {
            this.faceColors[i] = to;
          }
        }
      }
      retexture(from, to) {
        if (this.faceTextures) {
          for (let i = 0; i < this.faceCount; i++) {
            if (this.faceTextures[i] === from) {
              this.faceTextures[i] = to;
            }
          }
        }
      }
      mirror() {
        for (let i = 0; i < this.verticesCount; i++) {
          this.verticesZ[i] = -this.verticesZ[i];
        }
        for (let i = 0; i < this.faceCount; i++) {
          const temp = this.indices1[i];
          this.indices1[i] = this.indices3[i];
          this.indices3[i] = temp;
        }
        this.invalidate();
      }
      resize(resizeX, resizeY, resizeZ) {
        for (let i = 0; i < this.verticesCount; i++) {
          this.verticesX[i] = this.verticesX[i] * resizeX / 128 | 0;
          this.verticesY[i] = this.verticesY[i] * resizeY / 128 | 0;
          this.verticesZ[i] = this.verticesZ[i] * resizeZ / 128 | 0;
        }
        this.invalidate();
      }
      calculateVertexNormals() {
        if (!this.normals) {
          this.normals = new Array(this.usedVertexCount);
          for (let i = 0; i < this.usedVertexCount; i++) {
            this.normals[i] = new VertexNormal();
          }
          const verticesY = this.contourVerticesY || this.verticesY;
          for (let i = 0; i < this.faceCount; i++) {
            const var2 = this.indices1[i];
            const var3 = this.indices2[i];
            const var4 = this.indices3[i];
            const var5 = this.verticesX[var3] - this.verticesX[var2];
            const var6 = verticesY[var3] - verticesY[var2];
            const var7 = this.verticesZ[var3] - this.verticesZ[var2];
            const var8 = this.verticesX[var4] - this.verticesX[var2];
            const var9 = verticesY[var4] - verticesY[var2];
            const var10 = this.verticesZ[var4] - this.verticesZ[var2];
            let var11 = var6 * var10 - var9 * var7;
            let var12 = var7 * var8 - var10 * var5;
            let var13 = var5 * var9 - var8 * var6;
            while (var11 > 8192 || var12 > 8192 || var13 > 8192 || var11 < -8192 || var12 < -8192 || var13 < -8192) {
              var11 >>= 1;
              var12 >>= 1;
              var13 >>= 1;
            }
            let var14 = Math.sqrt(var11 * var11 + var12 * var12 + var13 * var13) | 0;
            if (var14 <= 0) {
              var14 = 1;
            }
            var11 = var11 * 256 / var14 | 0;
            var12 = var12 * 256 / var14 | 0;
            var13 = var13 * 256 / var14 | 0;
            let type;
            if (!this.faceRenderTypes) {
              type = 0;
            } else {
              type = this.faceRenderTypes[i];
            }
            if (type === 0) {
              let normal = this.normals[var2];
              normal.x += var11;
              normal.y += var12;
              normal.z += var13;
              normal.magnitude++;
              normal = this.normals[var3];
              normal.x += var11;
              normal.y += var12;
              normal.z += var13;
              normal.magnitude++;
              normal = this.normals[var4];
              normal.x += var11;
              normal.y += var12;
              normal.z += var13;
              normal.magnitude++;
            } else if (type === 1) {
              if (!this.faceNormals) {
                this.faceNormals = new Array(this.faceCount);
              }
              this.faceNormals[i] = new FaceNormal(var11, var12, var13);
            }
          }
        }
      }
      invalidate() {
        this.normals = void 0;
        this.mergedNormals = void 0;
        this.faceNormals = void 0;
        this.isBoundsCalculated = false;
      }
      calculateBounds() {
        if (!this.isBoundsCalculated) {
          this.height = 0;
          this.minHeight = 0;
          this.minX = 999999;
          this.maxX = -999999;
          this.minY = 999999;
          this.maxY = -999999;
          this.minZ = 99999;
          this.maxZ = -99999;
          const verticesY = this.contourVerticesY ?? this.verticesY;
          for (let i = 0; i < this.usedVertexCount; i++) {
            const vertX = this.verticesX[i];
            const vertY = verticesY[i];
            const vertZ = this.verticesZ[i];
            if (vertX < this.minX) {
              this.minX = vertX;
            }
            if (vertX > this.maxX) {
              this.maxX = vertX;
            }
            if (this.minY > vertY) {
              this.minY = vertY;
            }
            if (this.maxY < vertY) {
              this.maxY = vertY;
            }
            if (vertZ < this.minZ) {
              this.minZ = vertZ;
            }
            if (vertZ > this.maxZ) {
              this.maxZ = vertZ;
            }
            if (-vertY > this.height) {
              this.height = -vertY;
            }
            if (vertY > this.minHeight) {
              this.minHeight = vertY;
            }
          }
          this.isBoundsCalculated = true;
        }
      }
      light(textureLoader, ambient, contrast, lightX, lightY, lightZ) {
        this.calculateVertexNormals();
        if (!this.normals) {
          throw new Error("Failed to calculate normals. This should not be possible.");
        }
        const magnitude = Math.sqrt(lightZ * lightZ + lightX * lightX + lightY * lightY) | 0;
        const lightIntensity = magnitude * contrast >> 8;
        const model = new Model();
        model.faceColors1 = new Int32Array(this.faceCount);
        model.faceColors2 = new Int32Array(this.faceCount);
        model.faceColors3 = new Int32Array(this.faceCount);
        model.faceColors = this.faceColors;
        model.uvs = computeTextureCoords(textureLoader, this);
        if (this.faceTextures) {
          model.faceTextures = new Int16Array(this.faceCount);
          for (let i = 0; i < this.faceCount; i++) {
            const textureId = this.faceTextures[i];
            if (textureId !== -1 && textureLoader.isSd(textureId)) {
              model.faceTextures[i] = this.faceTextures[i];
            } else {
              model.faceTextures[i] = -1;
            }
          }
        } else {
          model.faceTextures = void 0;
        }
        if (this.textureFaceCount > 0 && this.textureCoords) {
          const textureCoords = new Int32Array(this.textureFaceCount);
          for (let i = 0; i < this.faceCount; i++) {
            if (this.textureCoords[i] !== -1) {
              textureCoords[this.textureCoords[i] & 255]++;
            }
          }
          model.texTriangleCount = 0;
          for (let i = 0; i < this.textureFaceCount; i++) {
            if (textureCoords[i] > 0 && this.textureRenderTypes[i] === 0) {
              model.texTriangleCount++;
            }
          }
          model.textureMappingP = new Int32Array(model.texTriangleCount);
          model.textureMappingM = new Int32Array(model.texTriangleCount);
          model.textureMappingN = new Int32Array(model.texTriangleCount);
          let mapIndex = 0;
          for (let i = 0; i < this.textureFaceCount; i++) {
            if (textureCoords[i] > 0 && this.textureRenderTypes[i] === 0) {
              model.textureMappingP[mapIndex] = this.textureMappingP[i] & 65535;
              model.textureMappingM[mapIndex] = this.textureMappingM[i] & 65535;
              model.textureMappingN[mapIndex] = this.textureMappingN[i] & 65535;
              textureCoords[i] = mapIndex++;
            } else {
              textureCoords[i] = -1;
            }
          }
          model.textureCoords = new Int8Array(this.faceCount);
          for (let i = 0; i < this.faceCount; i++) {
            if (this.textureCoords[i] !== -1) {
              model.textureCoords[i] = textureCoords[this.textureCoords[i] & 255];
            } else {
              model.textureCoords[i] = -1;
            }
          }
        }
        for (let i = 0; i < this.faceCount; i++) {
          let type;
          if (!this.faceRenderTypes) {
            type = 0;
          } else {
            type = this.faceRenderTypes[i];
          }
          let alpha;
          if (this.faceAlphas) {
            alpha = this.faceAlphas[i];
          } else {
            alpha = 0;
          }
          let texture;
          if (model.faceTextures) {
            texture = model.faceTextures[i];
          } else {
            texture = -1;
          }
          if (alpha === -2) {
            type = 3;
          }
          if (alpha === -1) {
            type = 2;
          }
          if (texture === -1) {
            if (type === 0) {
              const color = this.faceColors[i] & 65535;
              let normal;
              if (this.mergedNormals && this.mergedNormals[this.indices1[i]]) {
                normal = this.mergedNormals[this.indices1[i]];
              } else {
                normal = this.normals[this.indices1[i]];
              }
              let var14 = ambient + (lightY * normal.y + lightZ * normal.z + lightX * normal.x) / (lightIntensity * normal.magnitude) << 17;
              model.faceColors1[i] = var14 | _ModelData.adjustLightness(color, var14 >> 17);
              if (this.mergedNormals && this.mergedNormals[this.indices2[i]]) {
                normal = this.mergedNormals[this.indices2[i]];
              } else {
                normal = this.normals[this.indices2[i]];
              }
              var14 = ambient + (lightY * normal.y + lightZ * normal.z + lightX * normal.x) / (lightIntensity * normal.magnitude) << 17;
              model.faceColors2[i] = var14 | _ModelData.adjustLightness(color, var14 >> 17);
              if (this.mergedNormals && this.mergedNormals[this.indices3[i]]) {
                normal = this.mergedNormals[this.indices3[i]];
              } else {
                normal = this.normals[this.indices3[i]];
              }
              var14 = ambient + (lightY * normal.y + lightZ * normal.z + lightX * normal.x) / (lightIntensity * normal.magnitude) << 17;
              model.faceColors3[i] = var14 | _ModelData.adjustLightness(color, var14 >> 17);
            } else if (type === 1 && this.faceNormals) {
              const normal = this.faceNormals[i];
              const var14 = ambient + (lightY * normal.y + lightZ * normal.z + lightX * normal.x) / ((lightIntensity >> 1) + lightIntensity) << 17;
              model.faceColors1[i] = var14 | _ModelData.adjustLightness(this.faceColors[i] & 65535, var14 >> 17);
              model.faceColors3[i] = -1;
            } else if (type === 3) {
              model.faceColors1[i] = 128;
              model.faceColors3[i] = -1;
            } else {
              model.faceColors3[i] = -2;
            }
          } else if (type === 0) {
            let normal;
            if (this.mergedNormals && this.mergedNormals[this.indices1[i]]) {
              normal = this.mergedNormals[this.indices1[i]];
            } else {
              normal = this.normals[this.indices1[i]];
            }
            let var14 = ambient + (lightY * normal.y + lightZ * normal.z + lightX * normal.x) / (lightIntensity * normal.magnitude);
            model.faceColors1[i] = _ModelData.clampLightness(var14);
            if (this.mergedNormals && this.mergedNormals[this.indices2[i]]) {
              normal = this.mergedNormals[this.indices2[i]];
            } else {
              normal = this.normals[this.indices2[i]];
            }
            var14 = ambient + (lightY * normal.y + lightZ * normal.z + lightX * normal.x) / (lightIntensity * normal.magnitude);
            model.faceColors2[i] = _ModelData.clampLightness(var14);
            if (this.mergedNormals && this.mergedNormals[this.indices3[i]]) {
              normal = this.mergedNormals[this.indices3[i]];
            } else {
              normal = this.normals[this.indices3[i]];
            }
            var14 = ambient + (lightY * normal.y + lightZ * normal.z + lightX * normal.x) / (lightIntensity * normal.magnitude);
            model.faceColors3[i] = _ModelData.clampLightness(var14);
          } else if (type === 1 && this.faceNormals) {
            const normal = this.faceNormals[i];
            const var14 = ambient + (lightY * normal.y + lightZ * normal.z + lightX * normal.x) / ((lightIntensity >> 1) + lightIntensity);
            model.faceColors1[i] = _ModelData.clampLightness(var14);
            model.faceColors3[i] = -1;
          } else {
            model.faceColors3[i] = -2;
          }
        }
        this.computeAnimationTables();
        model.verticesCount = this.verticesCount;
        model.usedVertexCount = this.usedVertexCount;
        model.verticesX = this.verticesX;
        model.verticesY = this.verticesY;
        model.verticesZ = this.verticesZ;
        model.contourVerticesY = this.contourVerticesY;
        model.faceCount = this.faceCount;
        model.indices1 = this.indices1;
        model.indices2 = this.indices2;
        model.indices3 = this.indices3;
        model.faceRenderPriorities = this.faceRenderPriorities;
        model.faceAlphas = this.faceAlphas;
        model.priority = this.priority;
        model.vertexLabels = this.vertexLabels;
        model.faceLabels = this.faceLabels;
        model.textureScaleX = this.textureScaleX;
        model.textureScaleY = this.textureScaleY;
        model.textureScaleZ = this.textureScaleZ;
        model.textureRotation = this.textureRotation;
        model.textureDirection = this.textureDirection;
        model.textureSpeed = this.textureSpeed;
        model.textureTransU = this.textureTransU;
        model.textureTransV = this.textureTransV;
        model.animMayaGroups = this.animMayaGroups;
        model.animMayaScales = this.animMayaScales;
        return model;
      }
    };
  }
});

// {ROOT}/studies/tools-hd-era/client_bridge.ts
var import_node_fs = __toESM(require("node:fs"));
var import_node_path = __toESM(require("node:path"));
init_ByteBuffer();

// {TSPS_CHECKOUT}/client/common/utils/MathUtil.ts
var clamp = (num, min, max) => Math.min(Math.max(num, min), max);
function isPowerOfTwo(n) {
  return n === (-n & n);
}
function nextIntJagex(random, bound) {
  if (bound <= 0) {
    throw new Error("bound must be positive");
  }
  if (isPowerOfTwo(bound)) {
    return Number(BigInt(bound) * (BigInt(random.nextInt()) & 0xffffffffn) >> 32n);
  }
  const maxValue = -2147483648 - (4294967296 % bound | 0) | 0;
  let rndValue;
  do {
    rndValue = random.nextInt();
  } while (rndValue >= maxValue);
  return boundJagex(rndValue, bound);
}
function boundJagex(value, bound) {
  const i_78_ = value >> 31 & bound - 1;
  return i_78_ + (value + (value >>> 31)) % bound;
}

// {TSPS_CHECKOUT}/client/rs/texture/procedural/cache/ColourImageCache.ts
var import_denque = __toESM(require_denque());
var ColourImageCacheSlot = class {
  constructor(imageId, slotId) {
    this.imageId = imageId;
    this.slotId = slotId;
  }
};
var ColourImageCache = class _ColourImageCache {
  static {
    this.SLOT_USED = new ColourImageCacheSlot(0, 0);
  }
  constructor(slotCount, maxId, imageSize) {
    this.slotCount = slotCount;
    this.maxId = maxId;
    this.usageTracker = new import_denque.default();
    this.images = new Array(slotCount);
    for (let i = 0; i < slotCount; i++) {
      this.images[i] = new Array(3);
      for (let p = 0; p < 3; p++) {
        this.images[i][p] = new Int32Array(imageSize);
      }
    }
    this.slots = new Array(slotCount);
    this.usedSlots = 0;
    this.lastRequest = -1;
    this.dirty = false;
  }
  get(req) {
    if (this.slotCount === this.maxId) {
      this.dirty = this.slots[req] === void 0;
      this.slots[req] = _ColourImageCache.SLOT_USED;
      return this.images[req];
    } else if (this.slotCount === 1) {
      this.dirty = req !== this.lastRequest;
      this.lastRequest = req;
      return this.images[0];
    } else {
      let slot = this.slots[req];
      if (slot === void 0) {
        this.dirty = true;
        if (this.slotCount > this.usedSlots) {
          slot = new ColourImageCacheSlot(req, this.usedSlots);
          this.usedSlots++;
        } else {
          const oldSlot = this.usageTracker.pop();
          if (oldSlot) {
            slot = new ColourImageCacheSlot(req, oldSlot.slotId);
            delete this.slots[oldSlot.imageId];
          }
        }
        this.slots[req] = slot;
      } else {
        this.dirty = false;
      }
      for (let i = 0; i < this.usageTracker.length; i++) {
        if (this.usageTracker.peekAt(i) === slot) {
          this.usageTracker.removeOne(i);
          break;
        }
      }
      this.usageTracker.unshift(slot);
      return this.images[slot.slotId];
    }
  }
  getAll() {
    if (this.maxId !== this.slotCount) {
      throw new Error("Can only retrieve a full image cache");
    }
    for (let slot = 0; slot < this.slotCount; slot++) {
      this.slots[slot] = _ColourImageCache.SLOT_USED;
    }
    return this.images;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/cache/MonochromeImageCache.ts
var import_denque2 = __toESM(require_denque());
var MonochromeImageCacheSlot = class {
  constructor(imageId, slotId) {
    this.imageId = imageId;
    this.slotId = slotId;
  }
};
var MonochromeImageCache = class _MonochromeImageCache {
  static {
    this.SLOT_USED = new MonochromeImageCacheSlot(0, 0);
  }
  constructor(slotCount, maxId, imageSize) {
    this.slotCount = slotCount;
    this.maxId = maxId;
    this.usageTracker = new import_denque2.default();
    this.images = new Array(slotCount);
    for (let i = 0; i < slotCount; i++) {
      this.images[i] = new Int32Array(imageSize);
    }
    this.slots = new Array(slotCount);
    this.usedSlots = 0;
    this.lastRequest = -1;
    this.dirty = false;
  }
  get(req) {
    if (this.slotCount === this.maxId) {
      this.dirty = this.slots[req] === void 0;
      this.slots[req] = _MonochromeImageCache.SLOT_USED;
      return this.images[req];
    } else if (this.slotCount === 1) {
      this.dirty = req !== this.lastRequest;
      this.lastRequest = req;
      return this.images[0];
    } else {
      let slot = this.slots[req];
      if (slot === void 0) {
        this.dirty = true;
        if (this.slotCount > this.usedSlots) {
          slot = new MonochromeImageCacheSlot(req, this.usedSlots);
          this.usedSlots++;
        } else {
          const oldSlot = this.usageTracker.pop();
          if (oldSlot) {
            slot = new MonochromeImageCacheSlot(req, oldSlot.slotId);
            delete this.slots[oldSlot.imageId];
          }
        }
        this.slots[req] = slot;
      } else {
        this.dirty = false;
      }
      for (let i = 0; i < this.usageTracker.length; i++) {
        if (this.usageTracker.peekAt(i) === slot) {
          this.usageTracker.removeOne(i);
          break;
        }
      }
      this.usageTracker.unshift(slot);
      return this.images[slot.slotId];
    }
  }
  getAll() {
    if (this.maxId !== this.slotCount) {
      throw new Error("Can only retrieve a full image cache");
    }
    for (let slot = 0; slot < this.slotCount; slot++) {
      this.slots[slot] = _MonochromeImageCache.SLOT_USED;
    }
    return this.images;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/TextureOperation.ts
var TextureOperation = class {
  constructor(inputCount, isMonochrome) {
    this.id = -1;
    this.cacheSize = 0;
    this.isMonochrome = isMonochrome;
    this.inputs = new Array(inputCount);
  }
  decode(field, buffer) {
  }
  init() {
  }
  initCaches(textureGenerator, width, height) {
    const slotCount = this.cacheSize === 255 ? height : this.cacheSize;
    if (this.isMonochrome) {
      this.monochromeImageCache = new MonochromeImageCache(slotCount, height, width);
    } else {
      this.colourImageCache = new ColourImageCache(slotCount, height, width);
    }
  }
  clearCaches() {
    this.monochromeImageCache = void 0;
    this.colourImageCache = void 0;
  }
  getSpriteId() {
    return -1;
  }
  getTextureId() {
    return -1;
  }
  getMonochromeOutput(textureGenerator, line) {
    throw new Error("This operation does not have a monochrome output");
  }
  getColourOutput(textureGenerator, line) {
    throw new Error("This operation does not have a colour output");
  }
  getMonochromeInput(textureGenerator, n, line) {
    if (this.inputs[n].isMonochrome) {
      return this.inputs[n].getMonochromeOutput(textureGenerator, line);
    }
    return this.inputs[n].getColourOutput(textureGenerator, line)[0];
  }
  getColourInput(textureGenerator, n, line) {
    if (this.inputs[n].isMonochrome) {
      const monochromeOuputs = this.inputs[n].getMonochromeOutput(textureGenerator, line);
      const colourOutputs = new Array(3).fill(monochromeOuputs);
      return colourOutputs;
    }
    return this.inputs[n].getColourOutput(textureGenerator, line);
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/ArithmeticOperation.ts
var ArithmeticOperation = class extends TextureOperation {
  constructor() {
    super(2, false);
    this.operation = 6;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.operation = buffer.readUnsignedByte();
    } else if (field === 1) {
      this.isMonochrome = buffer.readUnsignedByte() === 1;
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const inputA = this.getMonochromeInput(textureGenerator, 0, line);
      const inputB = this.getMonochromeInput(textureGenerator, 1, line);
      switch (this.operation) {
        case 1:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            output[pixel] = inputA[pixel] + inputB[pixel];
          }
          break;
        case 2:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            output[pixel] = inputA[pixel] - inputB[pixel];
          }
          break;
        case 3:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            output[pixel] = inputB[pixel] * inputA[pixel] / 4096;
          }
          break;
        case 4:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const b = inputB[pixel];
            output[pixel] = b === 0 ? 4096 : inputA[pixel] * 4096 / b;
          }
          break;
        case 5:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            output[pixel] = 4096 - (4096 - inputA[pixel]) * (4096 - inputB[pixel]) / 4096;
          }
          break;
        case 6:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const b = inputB[pixel];
            output[pixel] = b < 2048 ? b * inputA[pixel] / 2048 : 4096 - ((4096 - inputA[pixel]) * (4096 - b) / 2048 | 0);
          }
          break;
        case 7:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const a = inputA[pixel];
            output[pixel] = a === 4096 ? 4096 : inputB[pixel] * 4096 / (4096 - a);
          }
          break;
        case 8:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const a = inputA[pixel];
            output[pixel] = a === 0 ? 0 : 4096 - (4096 - inputB[pixel]) * 4096 / a;
          }
          break;
        case 9:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const b = inputB[pixel];
            const a = inputA[pixel];
            output[pixel] = Math.min(a, b);
          }
          break;
        case 10:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const a = inputA[pixel];
            const b = inputB[pixel];
            output[pixel] = Math.max(a, b);
          }
          break;
        case 11:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const a = inputA[pixel];
            const b = inputB[pixel];
            output[pixel] = b < a ? a - b : b - a;
          }
          break;
        case 12:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const a = inputB[pixel];
            const b = inputA[pixel];
            output[pixel] = a + b - a * b / 2048;
          }
          break;
      }
    }
    return output;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const inputA = this.getColourInput(textureGenerator, 0, line);
      const inputB = this.getColourInput(textureGenerator, 1, line);
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      const inputAR = inputA[0];
      const inputAG = inputA[1];
      const inputAB = inputA[2];
      const inputBR = inputB[0];
      const inputBG = inputB[1];
      const inputBB = inputB[2];
      switch (this.operation) {
        case 1:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            outputR[pixel] = inputAR[pixel] + inputBR[pixel];
            outputG[pixel] = inputAG[pixel] + inputBG[pixel];
            outputB[pixel] = inputAB[pixel] + inputBB[pixel];
          }
          break;
        case 2:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            outputR[pixel] = inputAR[pixel] - inputBR[pixel];
            outputG[pixel] = inputAG[pixel] - inputBG[pixel];
            outputB[pixel] = inputAB[pixel] - inputBB[pixel];
          }
          break;
        case 3:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            outputR[pixel] = inputBR[pixel] * inputAR[pixel] / 4096;
            outputG[pixel] = inputBG[pixel] * inputAG[pixel] / 4096;
            outputB[pixel] = inputBB[pixel] * inputAB[pixel] / 4096;
          }
          break;
        case 4:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const bR = inputBR[pixel];
            const bG = inputBG[pixel];
            const bB = inputBB[pixel];
            outputR[pixel] = bR === 0 ? 4096 : inputAR[pixel] * 4096 / bR;
            outputG[pixel] = bG === 0 ? 4096 : inputAG[pixel] * 4096 / bG;
            outputB[pixel] = bB === 0 ? 4096 : inputAB[pixel] * 4096 / bB;
          }
          break;
        case 5:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            outputR[pixel] = 4096 - (4096 - inputAR[pixel]) * (4096 - inputBR[pixel]) / 4096;
            outputG[pixel] = 4096 - (4096 - inputAG[pixel]) * (4096 - inputBG[pixel]) / 4096;
            outputB[pixel] = 4096 - (4096 - inputAB[pixel]) * (4096 - inputBB[pixel]) / 4096;
          }
          break;
        case 6:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const bR = inputBR[pixel];
            const bG = inputBG[pixel];
            const bB = inputBB[pixel];
            outputR[pixel] = bR < 2048 ? bR * inputAR[pixel] / 2048 : 4096 - ((4096 - inputAR[pixel]) * (4096 - bR) / 2048 | 0);
            outputG[pixel] = bG < 2048 ? bG * inputAG[pixel] / 2048 : 4096 - ((4096 - inputAG[pixel]) * (4096 - bG) / 2048 | 0);
            outputB[pixel] = bB < 2048 ? bB * inputAB[pixel] / 2048 : 4096 - ((4096 - inputAB[pixel]) * (4096 - bB) / 2048 | 0);
          }
          break;
        case 7:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const aR = inputAR[pixel];
            const aG = inputAG[pixel];
            const aB = inputAB[pixel];
            outputR[pixel] = aR === 4096 ? 4096 : inputBR[pixel] * 4096 / (4096 - aR);
            outputG[pixel] = aG === 4096 ? 4096 : inputBG[pixel] * 4096 / (4096 - aG);
            outputB[pixel] = aB === 4096 ? 4096 : inputBB[pixel] * 4096 / (4096 - aB);
          }
          break;
        case 8:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const aR = inputAR[pixel];
            const aG = inputAG[pixel];
            const aB = inputAB[pixel];
            outputR[pixel] = aR === 0 ? 0 : 4096 - (4096 - inputBR[pixel]) * 4096 / aR;
            outputG[pixel] = aG === 0 ? 0 : 4096 - (4096 - inputBG[pixel]) * 4096 / aG;
            outputB[pixel] = aB === 0 ? 0 : 4096 - (4096 - inputBB[pixel]) * 4096 / aB;
          }
          break;
        case 9:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const aR = inputAR[pixel];
            const aG = inputAG[pixel];
            const aB = inputAB[pixel];
            const bR = inputBR[pixel];
            const bG = inputBG[pixel];
            const bB = inputBB[pixel];
            outputR[pixel] = Math.min(aR, bR);
            outputG[pixel] = Math.min(aG, bG);
            outputB[pixel] = Math.min(aB, bB);
          }
          break;
        case 10:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const aR = inputAR[pixel];
            const aG = inputAG[pixel];
            const aB = inputAB[pixel];
            const bR = inputBR[pixel];
            const bG = inputBG[pixel];
            const bB = inputBB[pixel];
            outputR[pixel] = Math.max(aR, bR);
            outputG[pixel] = Math.max(aG, bG);
            outputB[pixel] = Math.max(aB, bB);
          }
          break;
        case 11:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const aR = inputAR[pixel];
            const aG = inputAG[pixel];
            const aB = inputAB[pixel];
            const bR = inputBR[pixel];
            const bG = inputBG[pixel];
            const bB = inputBB[pixel];
            outputR[pixel] = bR < aR ? aR - bR : bR - aR;
            outputG[pixel] = bG < aG ? aG - bG : bG - aG;
            outputB[pixel] = bB < aB ? aB - bB : bB - aB;
          }
          break;
        case 12:
          for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
            const aR = inputBR[pixel];
            const aG = inputBG[pixel];
            const aB = inputBB[pixel];
            const bR = inputAR[pixel];
            const bG = inputAG[pixel];
            const bB = inputAB[pixel];
            outputR[pixel] = aR + bR - aR * bR / 2048;
            outputG[pixel] = aG + bG - aG * bG / 2048;
            outputB[pixel] = aB + bB - aB * bB / 2048;
          }
          break;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/BinaryOperation.ts
var BinaryOperation = class extends TextureOperation {
  constructor() {
    super(1, true);
    this.minValue = 0;
    this.maxValue = 4096;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.minValue = buffer.readUnsignedShort();
    } else if (field === 1) {
      this.maxValue = buffer.readUnsignedShort();
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const input = this.getMonochromeInput(textureGenerator, 0, line);
      for (let x = 0; x < textureGenerator.width; x++) {
        const value = input[x];
        output[x] = value >= this.minValue && value <= this.maxValue ? 4096 : 0;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/BlurOperation.ts
var BlurOperation = class extends TextureOperation {
  constructor() {
    super(1, false);
    this.hExtent = 1;
    this.vExtent = 1;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.hExtent = buffer.readUnsignedByte();
    } else if (field === 1) {
      this.vExtent = buffer.readUnsignedByte();
    } else if (field === 2) {
      this.isMonochrome = buffer.readUnsignedByte() === 1;
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const nPasses = 1 + (this.vExtent + this.vExtent);
      const invPasses = 65536 / nPasses | 0;
      const nPixels = 1 + this.hExtent + this.hExtent;
      const invPixels = 65536 / nPixels | 0;
      const passes = new Array(nPasses);
      for (let pass = -this.vExtent + line; pass <= line + this.vExtent; pass++) {
        const input = this.getMonochromeInput(
          textureGenerator,
          0,
          pass & textureGenerator.heightMask
        );
        const passOut = new Int32Array(textureGenerator.width);
        let sum = 0;
        for (let pixel = -this.hExtent; pixel <= this.hExtent; pixel++) {
          sum += input[pixel & textureGenerator.widthMask];
        }
        let ptr = 0;
        while (ptr < textureGenerator.width) {
          passOut[ptr] = sum * invPixels / 65536 | 0;
          sum -= input[ptr - this.hExtent & textureGenerator.widthMask];
          ptr++;
          sum += input[ptr + this.hExtent & textureGenerator.widthMask];
        }
        passes[this.vExtent - line + pass] = passOut;
      }
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        let sum = 0;
        for (let pass = 0; pass < nPasses; pass++) {
          sum += passes[pass][pixel];
        }
        output[pixel] = sum * invPasses / 65536 | 0;
      }
    }
    return output;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const nPasses = 1 + (this.vExtent + this.vExtent);
      const invPasses = 65536 / nPasses | 0;
      const nPixels = 1 + this.hExtent + this.hExtent;
      const invPixels = 65536 / nPixels | 0;
      const passes = new Array(nPasses);
      for (let pass = -this.vExtent + line; pass <= line + this.vExtent; pass++) {
        const input = this.getColourInput(
          textureGenerator,
          0,
          pass & textureGenerator.heightMask
        );
        const passOut = new Array(3);
        for (let i = 0; i < 3; i++) {
          passOut[i] = new Int32Array(textureGenerator.width);
        }
        let sumR = 0;
        let sumG = 0;
        let sumB = 0;
        for (let pixel = -this.hExtent; pixel <= this.hExtent; pixel++) {
          sumR += input[0][pixel & textureGenerator.widthMask];
          sumG += input[1][pixel & textureGenerator.widthMask];
          sumB += input[2][pixel & textureGenerator.widthMask];
        }
        let ptr = 0;
        while (ptr < textureGenerator.width) {
          passOut[0][ptr] = sumR * invPixels / 65536 | 0;
          passOut[1][ptr] = sumG * invPixels / 65536 | 0;
          passOut[2][ptr] = sumB * invPixels / 65536 | 0;
          sumR -= input[0][ptr - this.hExtent & textureGenerator.widthMask];
          sumG -= input[1][ptr - this.hExtent & textureGenerator.widthMask];
          sumB -= input[2][ptr - this.hExtent & textureGenerator.widthMask];
          ptr++;
          sumR += input[0][ptr + this.hExtent & textureGenerator.widthMask];
          sumG += input[1][ptr + this.hExtent & textureGenerator.widthMask];
          sumB += input[2][ptr + this.hExtent & textureGenerator.widthMask];
        }
        passes[this.vExtent + pass - line] = passOut;
      }
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        let sumR = 0;
        let sumG = 0;
        let sumB = 0;
        for (let pass = 0; pass < nPasses; pass++) {
          sumR += passes[pass][0][pixel];
          sumG += passes[pass][1][pixel];
          sumB += passes[pass][2][pixel];
        }
        output[0][pixel] = sumR * invPasses / 65536 | 0;
        output[1][pixel] = sumG * invPasses / 65536 | 0;
        output[2][pixel] = sumB * invPasses / 65536 | 0;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/BricksOperation.ts
var import_java_random = __toESM(require_lib());
var BricksOperation = class extends TextureOperation {
  constructor() {
    super(0, true);
    this.field0 = 4;
    this.seed = 8;
    this.field2 = 409;
    this.field3 = 204;
    this.field4 = 1024;
    this.field5 = 0;
    this.field6 = 81;
    this.field7 = 1024;
    this.halfField6 = 0;
    this.ratio0 = 0;
    this.ratio1 = 0;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.field0 = buffer.readUnsignedByte();
    } else if (field === 1) {
      this.seed = buffer.readUnsignedByte();
    } else if (field === 2) {
      this.field2 = buffer.readUnsignedShort();
    } else if (field === 3) {
      this.field3 = buffer.readUnsignedShort();
    } else if (field === 4) {
      this.field4 = buffer.readUnsignedShort();
    } else if (field === 5) {
      this.field5 = buffer.readUnsignedShort();
    } else if (field === 6) {
      this.field6 = buffer.readUnsignedShort();
    } else if (field === 7) {
      this.field7 = buffer.readUnsignedShort();
    }
  }
  init() {
    this.table0 = new Array(this.seed);
    this.table1 = new Array(this.seed);
    for (let i = 0; i < this.seed; i++) {
      this.table0[i] = new Int32Array(this.field0);
      this.table1[i] = new Int32Array(this.field0 + 1);
    }
    this.table2 = new Int32Array(this.seed + 1);
    const random = new import_java_random.default(this.seed);
    this.halfField6 = this.field6 / 2 | 0;
    this.ratio0 = 4096 / this.field0 | 0;
    const halfR0 = this.ratio0 / 2 | 0;
    this.ratio1 = 4096 / this.seed | 0;
    const halfR1 = this.ratio1 / 2 | 0;
    this.table2[0] = 0;
    for (let x = 0; x < this.seed; x++) {
      if (x > 0) {
        let value = this.ratio1;
        const randomValue = (nextIntJagex(random, 4096) - 2048) * this.field3 >> 12;
        value += randomValue * halfR1 >> 12;
        this.table2[x] = value + this.table2[x - 1];
      }
      this.table1[x][0] = 0;
      for (let y = 0; y < this.field0; y++) {
        if (y > 0) {
          let value = this.ratio0;
          const randomValue = (nextIntJagex(random, 4096) - 2048) * this.field2 >> 12;
          value += randomValue * halfR0 >> 12;
          this.table1[x][y] = this.table1[x][y - 1] + value;
        }
        this.table0[x][y] = this.field7 > 0 ? 4096 - nextIntJagex(random, this.field7) : 4096;
      }
      this.table1[x][this.field0] = 4096;
    }
    this.table2[this.seed] = 4096;
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      let index0 = 0;
      let value0 = this.field5 + textureGenerator.verticalGradient[line];
      for (; value0 < 0; value0 += 4096) ;
      for (; value0 > 4096; value0 -= 4096) ;
      for (; index0 < this.seed; index0++) {
        if (value0 < this.table2[index0]) {
          break;
        }
      }
      const tv0 = this.table2[index0 - 1];
      const tv1 = this.table2[index0];
      if (value0 > this.halfField6 + tv0 && value0 < tv1 - this.halfField6) {
        for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
          const f4 = index0 % 2 !== 0 ? -this.field4 : this.field4;
          let index1 = 0;
          let value1 = (this.ratio0 * f4 >> 12) + textureGenerator.horizontalGradient[pixel];
          for (; value1 < 0; value1 += 4096) ;
          for (; value1 > 4096; value1 -= 4096) ;
          for (; index1 < this.field0; index1++) {
            if (value1 < this.table1[index0 - 1][index1]) {
              break;
            }
          }
          const tv2 = this.table1[index0 - 1][index1 - 1];
          const tv3 = this.table1[index0 - 1][index1];
          if (tv2 + this.halfField6 < value1 && value1 < tv3 - this.halfField6) {
            output[pixel] = this.table0[index0 - 1][index1 - 1];
          } else {
            output[pixel] = 0;
          }
        }
      } else {
        output.fill(0, 0, textureGenerator.width);
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/BrightnessOperation.ts
var BrightnessOperation = class extends TextureOperation {
  constructor() {
    super(1, false);
    this.maxValue = 409;
    this.redFactor = 4096;
    this.greenFactor = 4096;
    this.blueFactor = 4096;
    this.colorDelta = new Int32Array(3);
  }
  decode(field, buffer) {
    if (field === 0) {
      this.maxValue = buffer.readUnsignedShort();
    } else if (field === 1) {
      this.blueFactor = buffer.readUnsignedShort();
    } else if (field === 2) {
      this.greenFactor = buffer.readUnsignedShort();
    } else if (field === 3) {
      this.redFactor = buffer.readUnsignedShort();
    } else if (field === 4) {
      const rgb = buffer.readMedium();
      this.colorDelta[0] = (rgb & 16711680) << 4;
      this.colorDelta[1] = rgb >> 4 & 4080;
      this.colorDelta[2] = rgb >> 12 & 0;
    }
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const input = this.getColourInput(textureGenerator, 0, line);
      const inputR = input[0];
      const inputG = input[1];
      const inputB = input[2];
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      for (let x = 0; x < textureGenerator.width; x++) {
        const r = inputR[x];
        let absR = r - this.colorDelta[0];
        if (absR < 0) {
          absR = -absR;
        }
        if (absR <= this.maxValue) {
          const g = inputG[x];
          let absG = g - this.colorDelta[1];
          if (absG < 0) {
            absG = -absG;
          }
          if (absG <= this.maxValue) {
            const b = inputB[x];
            let absB = b - this.colorDelta[2];
            if (absB < 0) {
              absB = -absB;
            }
            if (absB <= this.maxValue) {
              outputR[x] = r * this.redFactor >> 12;
              outputG[x] = g * this.greenFactor >> 12;
              outputB[x] = b * this.blueFactor >> 12;
            } else {
              outputR[x] = r;
              outputG[x] = g;
              outputB[x] = b;
            }
          } else {
            outputR[x] = r;
            outputG[x] = g;
            outputB[x] = inputB[x];
          }
        } else {
          outputR[x] = r;
          outputG[x] = inputG[x];
          outputB[x] = inputB[x];
        }
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/ClampOperation.ts
var ClampOperation = class extends TextureOperation {
  constructor() {
    super(1, false);
    this.min = 0;
    this.max = 4096;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.min = buffer.readUnsignedShort();
    } else if (field === 1) {
      this.max = buffer.readUnsignedShort();
    } else if (field === 2) {
      this.isMonochrome = buffer.readUnsignedByte() === 1;
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const input = this.getMonochromeInput(textureGenerator, 0, line);
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        const value = input[pixel];
        output[pixel] = clamp(value, this.min, this.max);
      }
    }
    return output;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const input = this.getColourInput(textureGenerator, 0, line);
      const inputR = input[0];
      const inputG = input[1];
      const inputB = input[2];
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        const valueR = inputR[pixel];
        const valueG = inputG[pixel];
        const valueB = inputB[pixel];
        outputR[pixel] = clamp(valueR, this.min, this.max);
        outputG[pixel] = clamp(valueG, this.min, this.max);
        outputB[pixel] = clamp(valueB, this.min, this.max);
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/ColorEdgeDetectorOperation.ts
var ColorEdgeDetectorOperation = class extends TextureOperation {
  constructor() {
    super(1, false);
    this.multiplier = 4096;
    this.field1 = true;
  }
  decode(field, buffer) {
    if (field === 1) {
      this.multiplier = buffer.readUnsignedShort();
    } else if (field === 2) {
      this.field1 = buffer.readUnsignedByte() === 1;
    }
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const prevInput = this.getMonochromeInput(
        textureGenerator,
        0,
        line - 1 & textureGenerator.heightMask
      );
      const input = this.getMonochromeInput(textureGenerator, 0, line);
      const nextInput = this.getMonochromeInput(
        textureGenerator,
        0,
        line + 1 & textureGenerator.heightMask
      );
      const inputR = input[0];
      const inputG = input[1];
      const inputB = input[2];
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      for (let x = 0; x < textureGenerator.width; x++) {
        const dy = this.multiplier * (nextInput[x] - prevInput[x]);
        const dx = this.multiplier * (input[x + 1 & textureGenerator.widthMask] - input[x - 1 & textureGenerator.widthMask]);
        const dy0 = dy >> 12;
        const dx0 = dx >> 12;
        const dySquared = dy0 * dy0 >> 12;
        const dxSquared = dx0 * dx0 >> 12;
        const local137 = Math.sqrt((dySquared + dxSquared + 4096) / 4096) * 4096 | 0;
        let red;
        let green;
        let blue;
        if (local137 == 0) {
          red = 0;
          green = 0;
          blue = 0;
        } else {
          red = dx / local137 | 0;
          green = dy / local137 | 0;
          blue = 16777216 / local137 | 0;
        }
        if (this.field1) {
          red = (red >> 1) + 2048;
          green = (green >> 1) + 2048;
          blue = (blue >> 1) + 2048;
        }
        outputR[x] = red;
        outputG[x] = green;
        outputB[x] = blue;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/ColourStripOperation.ts
var ColourStripOperation = class extends TextureOperation {
  constructor() {
    super(1, false);
    this.colourR = 4096;
    this.colourG = 4096;
    this.colourB = 4096;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.colourR = buffer.readUnsignedShort();
    } else if (field === 1) {
      this.colourG = buffer.readUnsignedShort();
    } else if (field === 2) {
      this.colourB = buffer.readUnsignedShort();
    }
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const input = this.getColourInput(textureGenerator, 0, line);
      const inputR = input[0];
      const inputG = input[1];
      const inputB = input[2];
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        const valueR = inputR[pixel];
        const valueG = inputG[pixel];
        const valueB = inputB[pixel];
        if (valueR !== valueB || valueB !== valueG) {
          outputR[pixel] = this.colourR;
          outputG[pixel] = this.colourG;
          outputB[pixel] = this.colourB;
        } else {
          outputR[pixel] = this.colourR * valueR >> 12;
          outputG[pixel] = this.colourG * valueG >> 12;
          outputB[pixel] = this.colourB * valueB >> 12;
        }
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/ConstantColourOperation.ts
var ConstantColourOperation = class extends TextureOperation {
  constructor(rgb = 0) {
    super(0, false);
    this.constantR = 0;
    this.constantG = 0;
    this.constantB = 0;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.setConstant(buffer.readMedium());
    }
  }
  setConstant(rgb) {
    this.constantR = (rgb >> 16 & 255) * 16;
    this.constantG = (rgb >> 8 & 255) * 16;
    this.constantB = (rgb & 255) * 16;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        outputR[pixel] = this.constantR;
        outputG[pixel] = this.constantG;
        outputB[pixel] = this.constantB;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/ConstantMonochromeOperation.ts
var ConstantMonochromeOperation = class extends TextureOperation {
  constructor(constant = 4096) {
    super(0, true);
    this.constant = constant;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.constant = (buffer.readUnsignedByte() << 12) / 255 | 0;
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      output.fill(this.constant, 0, textureGenerator.width);
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/TextureGenerator.ts
var import_java_random2 = __toESM(require_lib());

// {TSPS_CHECKOUT}/client/rs/sprite/SpriteLoader.ts
init_ByteBuffer();

// {TSPS_CHECKOUT}/client/rs/graphics/Rasterizer2D.ts
var Rasterizer2D = class _Rasterizer2D {
  static setRaster(pixels, width, height) {
    _Rasterizer2D.pixels = pixels;
    _Rasterizer2D.width = width;
    _Rasterizer2D.height = height;
    _Rasterizer2D.setClip(0, 0, width, height);
  }
  static setClip(x, y, width, height) {
    if (x < 0) {
      x = 0;
    }
    if (y < 0) {
      y = 0;
    }
    if (width > _Rasterizer2D.width) {
      width = _Rasterizer2D.width;
    }
    if (height > _Rasterizer2D.height) {
      height = _Rasterizer2D.height;
    }
    _Rasterizer2D.xClipStart = x;
    _Rasterizer2D.yClipStart = y;
    _Rasterizer2D.xClipEnd = width;
    _Rasterizer2D.yClipEnd = height;
  }
  static fillRectangle(x, y, width, height, rgb) {
    if (x < _Rasterizer2D.xClipStart) {
      width -= _Rasterizer2D.xClipStart - x;
      x = _Rasterizer2D.xClipStart;
    }
    if (y < _Rasterizer2D.yClipStart) {
      height -= _Rasterizer2D.yClipStart - y;
      y = _Rasterizer2D.yClipStart;
    }
    if (x + width > _Rasterizer2D.xClipEnd) {
      width = _Rasterizer2D.xClipEnd - x;
    }
    if (height + y > _Rasterizer2D.yClipEnd) {
      height = _Rasterizer2D.yClipEnd - y;
    }
    const widthOffset = _Rasterizer2D.width - width;
    let offset = x + _Rasterizer2D.width * y;
    for (let h = -height; h < 0; h++) {
      for (let w = -width; w < 0; w++) {
        _Rasterizer2D.pixels[offset++] = rgb;
      }
      offset += widthOffset;
    }
  }
};

// {TSPS_CHECKOUT}/client/rs/sprite/IndexedSprite.ts
var IndexedSprite = class _IndexedSprite {
  static raster(pixels, spritePixels, palette, startX, startY, width, height, endX, endY) {
    let var9 = -(width >> 2);
    width = -(width & 3);
    for (let var10 = -height; var10 < 0; var10++) {
      for (let var11 = var9; var11 < 0; var11++) {
        let p = spritePixels[startX++];
        if (p !== 0) {
          pixels[startY++] = palette[p & 255];
        } else {
          startY++;
        }
        p = spritePixels[startX++];
        if (p !== 0) {
          pixels[startY++] = palette[p & 255];
        } else {
          startY++;
        }
        p = spritePixels[startX++];
        if (p !== 0) {
          pixels[startY++] = palette[p & 255];
        } else {
          startY++;
        }
        p = spritePixels[startX++];
        if (p !== 0) {
          pixels[startY++] = palette[p & 255];
        } else {
          startY++;
        }
      }
      for (let var11 = width; var11 < 0; var11++) {
        let var12 = spritePixels[startX++];
        if (var12 !== 0) {
          pixels[startY++] = palette[var12 & 255];
        } else {
          startY++;
        }
      }
      startY += endX;
      startX += endY;
    }
  }
  normalize() {
    if (this.subWidth !== this.width || this.subHeight !== this.height) {
      const pixels = new Uint8Array(this.width * this.height);
      let index = 0;
      for (let y = 0; y < this.subHeight; y++) {
        for (let x = 0; x < this.subWidth; x++) {
          pixels[x + (y + this.yOffset) * this.width + this.xOffset] = this.pixels[index++];
        }
      }
      this.pixels = pixels;
      this.subWidth = this.width;
      this.subHeight = this.height;
      this.xOffset = 0;
      this.yOffset = 0;
    }
  }
  shiftColors(rOffset, gOffset, bOffset) {
    for (let i = 0; i < this.palette.length; i++) {
      let r = this.palette[i] >> 16 & 255;
      r += rOffset;
      if (r < 0) {
        r = 0;
      } else if (r > 255) {
        r = 255;
      }
      let g = this.palette[i] >> 8 & 255;
      g += gOffset;
      if (g < 0) {
        g = 0;
      } else if (g > 255) {
        g = 255;
      }
      let b = this.palette[i] & 255;
      b += bOffset;
      if (b < 0) {
        b = 0;
      } else if (b > 255) {
        b = 255;
      }
      this.palette[i] = b + (g << 8) + (r << 16);
    }
  }
  getPixelsRgb() {
    const dstWidth = this.width;
    const dst = new Int32Array(this.width * this.height);
    for (let y = 0; y < this.height; y++) {
      let srcIndex = y * this.width;
      let dstIndex = this.xOffset + (y + this.yOffset) * dstWidth;
      for (let x = 0; x < this.width; x++) {
        const rgb = this.palette[this.pixels[srcIndex++] & 255];
        if (rgb !== 0) {
          dst[dstIndex++] = ~16777215 | rgb;
        } else {
          dst[dstIndex++] = 0;
        }
      }
    }
    return dst;
  }
  getCanvas() {
    const canvas = new OffscreenCanvas(this.width, this.height);
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    const imageData = ctx.createImageData(this.width, this.height);
    for (let i = 0; i < this.pixels.length; i++) {
      const rgb = this.palette[this.pixels[i] & 255];
      if (rgb !== 0) {
        imageData.data[i * 4] = rgb >> 16 & 255;
        imageData.data[i * 4 + 1] = rgb >> 8 & 255;
        imageData.data[i * 4 + 2] = rgb & 255;
        imageData.data[i * 4 + 3] = 255;
      }
    }
    ctx.putImageData(imageData, 0, 0);
    return canvas;
  }
  drawAt(x, y) {
    x += this.xOffset;
    y += this.yOffset;
    let startY = x + y * Rasterizer2D.width;
    let startX = 0;
    let height = this.subHeight;
    let width = this.subWidth;
    let endX = Rasterizer2D.width - width;
    let endY = 0;
    if (y < Rasterizer2D.yClipStart) {
      const var9 = Rasterizer2D.yClipStart - y;
      height -= var9;
      y = Rasterizer2D.yClipStart;
      startX += var9 * width;
      startY += var9 * Rasterizer2D.width;
    }
    if (height + y > Rasterizer2D.yClipEnd) {
      height -= height + y - Rasterizer2D.yClipEnd;
    }
    if (x < Rasterizer2D.xClipStart) {
      const var9 = Rasterizer2D.xClipStart - x;
      width -= var9;
      x = Rasterizer2D.xClipStart;
      startX += var9;
      startY += var9;
      endY += var9;
      endX += var9;
    }
    if (width + x > Rasterizer2D.xClipEnd) {
      const var9 = width + x - Rasterizer2D.xClipEnd;
      width -= var9;
      endY += var9;
      endX += var9;
    }
    if (width > 0 && height > 0) {
      _IndexedSprite.raster(
        Rasterizer2D.pixels,
        this.pixels,
        this.palette,
        startX,
        startY,
        width,
        height,
        endX,
        endY
      );
    }
  }
};

// {TSPS_CHECKOUT}/client/rs/sprite/SpriteLoader.ts
var SpriteLoader = class _SpriteLoader {
  static {
    this.spriteCount = 0;
  }
  static {
    this.width = 0;
  }
  static {
    this.height = 0;
  }
  static load(data) {
    const buffer = new ByteBuffer(data);
    buffer.offset = data.length - 2;
    _SpriteLoader.spriteCount = buffer.readUnsignedShort();
    _SpriteLoader.xOffsets = new Int32Array(_SpriteLoader.spriteCount);
    _SpriteLoader.yOffsets = new Int32Array(_SpriteLoader.spriteCount);
    _SpriteLoader.widths = new Int32Array(_SpriteLoader.spriteCount);
    _SpriteLoader.heights = new Int32Array(_SpriteLoader.spriteCount);
    _SpriteLoader.pixels = new Array(_SpriteLoader.spriteCount);
    _SpriteLoader.alphas = new Array(_SpriteLoader.spriteCount);
    buffer.offset = data.length - 7 - _SpriteLoader.spriteCount * 8;
    _SpriteLoader.width = buffer.readUnsignedShort();
    _SpriteLoader.height = buffer.readUnsignedShort();
    const paletteSize = (buffer.readUnsignedByte() & 255) + 1;
    for (let i = 0; i < _SpriteLoader.spriteCount; i++) {
      _SpriteLoader.xOffsets[i] = buffer.readUnsignedShort();
    }
    for (let i = 0; i < _SpriteLoader.spriteCount; i++) {
      _SpriteLoader.yOffsets[i] = buffer.readUnsignedShort();
    }
    for (let i = 0; i < _SpriteLoader.spriteCount; i++) {
      _SpriteLoader.widths[i] = buffer.readUnsignedShort();
    }
    for (let i = 0; i < _SpriteLoader.spriteCount; i++) {
      _SpriteLoader.heights[i] = buffer.readUnsignedShort();
    }
    buffer.offset = data.length - 7 - _SpriteLoader.spriteCount * 8 - (paletteSize - 1) * 3;
    _SpriteLoader.palette = new Int32Array(paletteSize);
    for (let i = 1; i < paletteSize; i++) {
      _SpriteLoader.palette[i] = buffer.readMedium();
      if (_SpriteLoader.palette[i] === 0) {
        _SpriteLoader.palette[i] = 1;
      }
    }
    buffer.offset = 0;
    for (let i = 0; i < _SpriteLoader.spriteCount; i++) {
      const width = _SpriteLoader.widths[i];
      const height = _SpriteLoader.heights[i];
      const pixelCount = width * height;
      const pixels = _SpriteLoader.pixels[i] = new Uint8Array(pixelCount);
      const flags = buffer.readUnsignedByte();
      const isColumnWise = (flags & 1) !== 0;
      const hasAlpha = (flags & 2) !== 0;
      if (!isColumnWise) {
        for (let pi = 0; pi < pixelCount; pi++) {
          pixels[pi] = buffer.readByte();
        }
      } else {
        for (let x = 0; x < width; x++) {
          for (let y = 0; y < height; y++) {
            pixels[x + y * width] = buffer.readByte();
          }
        }
      }
      if (hasAlpha) {
        const alphaData = new Uint8Array(pixelCount);
        if (!isColumnWise) {
          for (let pi = 0; pi < pixelCount; pi++) {
            alphaData[pi] = buffer.readUnsignedByte();
          }
        } else {
          for (let x = 0; x < width; x++) {
            for (let y = 0; y < height; y++) {
              alphaData[x + y * width] = buffer.readUnsignedByte();
            }
          }
        }
        _SpriteLoader.alphas[i] = alphaData;
      }
    }
  }
  static reset() {
    _SpriteLoader.xOffsets = void 0;
    _SpriteLoader.yOffsets = void 0;
    _SpriteLoader.widths = void 0;
    _SpriteLoader.heights = void 0;
    _SpriteLoader.palette = void 0;
    _SpriteLoader.pixels = void 0;
    _SpriteLoader.alphas = void 0;
  }
  static loadFromIndex(spriteIndex, id) {
    if (id == null || id < 0) return false;
    let file = void 0;
    const archiveId = id >>> 16 & 65535;
    const fileId = id & 65535;
    try {
      if (archiveId !== 0 || fileId !== 0) {
        file = spriteIndex.getFile(archiveId, fileId);
      }
    } catch {
    }
    if (!file) {
      try {
        file = spriteIndex.getFile(id, 0);
      } catch {
      }
    }
    if (file) {
      _SpriteLoader.load(file.data);
      return true;
    }
    return false;
  }
  static loadIndexedSpriteDat(archive, name, offset) {
    return this.loadIndexedSpriteDatId(archive, archive.getFileId(name + ".dat"), offset);
  }
  static loadIndexedSpriteDatId(archive, id, offset) {
    const dataFile = archive.getFile(id);
    const indexFile = archive.getFileNamed("index.dat");
    if (!dataFile) {
      throw new Error(id + " sprite not found");
    }
    if (!indexFile) {
      throw new Error("index.dat not found");
    }
    const dataBuffer = new ByteBuffer(dataFile.data);
    const indexBuffer = new ByteBuffer(indexFile.data);
    indexBuffer.offset = dataBuffer.readUnsignedShort();
    const sprite = new IndexedSprite();
    sprite.width = indexBuffer.readUnsignedShort();
    sprite.height = indexBuffer.readUnsignedShort();
    let paletteSize = indexBuffer.readUnsignedByte();
    sprite.palette = new Int32Array(paletteSize);
    for (let i = 0; i < paletteSize - 1; i++) {
      sprite.palette[i + 1] = indexBuffer.readMedium();
    }
    for (let i = 0; i < offset; i++) {
      indexBuffer.offset += 2;
      dataBuffer.offset += indexBuffer.readUnsignedShort() * indexBuffer.readUnsignedShort();
      indexBuffer.offset++;
    }
    sprite.xOffset = indexBuffer.readUnsignedByte();
    sprite.yOffset = indexBuffer.readUnsignedByte();
    sprite.subWidth = indexBuffer.readUnsignedShort();
    sprite.subHeight = indexBuffer.readUnsignedShort();
    const pixelCount = sprite.subWidth * sprite.subHeight;
    sprite.pixels = new Uint8Array(pixelCount);
    const type = indexBuffer.readUnsignedByte();
    if (type === 0) {
      for (let i = 0; i < pixelCount; i++) {
        sprite.pixels[i] = dataBuffer.readByte();
      }
    } else if (type === 1) {
      for (let x = 0; x < sprite.subWidth; x++) {
        for (let y = 0; y < sprite.subHeight; y++) {
          sprite.pixels[x + y * sprite.subWidth] = dataBuffer.readByte();
        }
      }
    }
    return sprite;
  }
  static loadIntoIndexedSprite(spriteIndex, id) {
    if (_SpriteLoader.loadFromIndex(spriteIndex, id) && _SpriteLoader.xOffsets && _SpriteLoader.yOffsets && _SpriteLoader.widths && _SpriteLoader.heights && _SpriteLoader.palette && _SpriteLoader.pixels) {
      const sprite = new IndexedSprite();
      sprite.width = _SpriteLoader.width;
      sprite.height = _SpriteLoader.height;
      sprite.xOffset = _SpriteLoader.xOffsets[0];
      sprite.yOffset = _SpriteLoader.yOffsets[0];
      sprite.subWidth = _SpriteLoader.widths[0];
      sprite.subHeight = _SpriteLoader.heights[0];
      sprite.palette = _SpriteLoader.palette;
      sprite.pixels = _SpriteLoader.pixels[0];
      sprite.alpha = _SpriteLoader.alphas?.[0];
      _SpriteLoader.reset();
      return sprite;
    }
    return void 0;
  }
  static loadIntoIndexedSprites(spriteIndex, id) {
    if (_SpriteLoader.loadFromIndex(spriteIndex, id) && _SpriteLoader.xOffsets && _SpriteLoader.yOffsets && _SpriteLoader.widths && _SpriteLoader.heights && _SpriteLoader.palette && _SpriteLoader.pixels) {
      const sprites = new Array(_SpriteLoader.spriteCount);
      for (let i = 0; i < _SpriteLoader.spriteCount; i++) {
        const sprite = sprites[i] = new IndexedSprite();
        sprite.width = _SpriteLoader.width;
        sprite.height = _SpriteLoader.height;
        sprite.xOffset = _SpriteLoader.xOffsets[i];
        sprite.yOffset = _SpriteLoader.yOffsets[i];
        sprite.subWidth = _SpriteLoader.widths[i];
        sprite.subHeight = _SpriteLoader.heights[i];
        sprite.palette = _SpriteLoader.palette;
        sprite.pixels = _SpriteLoader.pixels[i];
        sprite.alpha = _SpriteLoader.alphas?.[i];
      }
      _SpriteLoader.reset();
      return sprites;
    }
    return void 0;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/TextureGenerator.ts
var TextureGenerator = class _TextureGenerator {
  constructor(spriteIndex, textureLoader) {
    this.width = 0;
    this.height = 0;
    this.widthTimes32 = 0;
    this.widthMask = 0;
    this.heightMask = 0;
    this.brightnessTable = new Int32Array(256);
    this.brightness = -1;
    this.isTransparent = false;
    this.debug = false;
    this.spriteIndex = spriteIndex;
    this.textureLoader = textureLoader;
  }
  static {
    this.permutationCache = /* @__PURE__ */ new Map();
  }
  static initTrig() {
    _TextureGenerator.SINE = new Int32Array(256);
    _TextureGenerator.COSINE = new Int32Array(256);
    for (let i = 0; i < 256; i++) {
      const d2 = i / 255 * 6.283185307179586;
      _TextureGenerator.SINE[i] = Math.sin(d2) * 4096;
      _TextureGenerator.COSINE[i] = Math.cos(d2) * 4096;
    }
  }
  static initInverseSquareRoot() {
    _TextureGenerator.INVERSE_SQUARE_ROOT = new Int8Array(32896);
    let i = 0;
    for (let x = 0; x < 256; x++) {
      for (let y = 0; y <= x; y++) {
        _TextureGenerator.INVERSE_SQUARE_ROOT[i++] = 255 / Math.sqrt(Math.fround((x * x + y * y + 65535) / 65535)) | 0;
      }
    }
  }
  static init() {
    _TextureGenerator.initTrig();
    _TextureGenerator.initInverseSquareRoot();
  }
  init(width, height) {
    this.isTransparent = false;
    if (this.width !== width) {
      this.horizontalGradient = new Int32Array(width);
      for (let i = 0; i < width; i++) {
        this.horizontalGradient[i] = (i << 12) / width;
      }
      this.widthMask = width - 1;
      this.width = width;
      this.widthTimes32 = width * 32;
    }
    if (this.height !== height) {
      if (height !== this.width) {
        this.verticalGradient = new Int32Array(height);
        for (let i = 0; i < height; i++) {
          this.verticalGradient[i] = (i << 12) / height;
        }
      } else {
        this.verticalGradient = this.horizontalGradient;
      }
      this.heightMask = height - 1;
      this.height = height;
    }
  }
  initBrightness(brightness) {
    if (this.brightness !== brightness) {
      for (let i = 0; i < this.brightnessTable.length; i++) {
        const v = Math.pow(i / 255, brightness) * 255 | 0;
        this.brightnessTable[i] = Math.min(v, 255);
      }
      this.brightness = brightness;
    }
  }
  static initPermutations(seed) {
    const cached = _TextureGenerator.permutationCache.get(seed);
    if (cached) {
      return cached;
    }
    const permutations = new Int8Array(512);
    const random = new import_java_random2.default(seed);
    for (let i = 0; i < 255; i++) {
      permutations[i] = i;
    }
    for (let i = 0; i < 255; i++) {
      const index0 = 255 - i;
      const index1 = nextIntJagex(random, index0);
      const perm1 = permutations[index1];
      permutations[index1] = permutations[index0];
      permutations[index0] = permutations[511 - i] = perm1;
    }
    _TextureGenerator.permutationCache.set(seed, permutations);
    return permutations;
  }
  loadSprite(spriteId) {
    const sprite = SpriteLoader.loadIntoIndexedSprite(this.spriteIndex, spriteId);
    if (!sprite) {
      throw new Error("Sprite not found: " + spriteId);
    }
    return sprite;
  }
};
TextureGenerator.init();

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/CurveOperation.ts
var CurveOperation = class extends TextureOperation {
  constructor() {
    super(1, true);
    this.interpMode = 0;
    this.table = new Int16Array(257);
  }
  decode(field, buffer) {
    if (field === 0) {
      this.interpMode = buffer.readUnsignedByte();
      const markerCount = buffer.readUnsignedByte();
      this.markers = new Array(markerCount);
      for (let i = 0; i < markerCount; i++) {
        const marker = this.markers[i] = new Array(2);
        marker[0] = buffer.readUnsignedShort();
        marker[1] = buffer.readUnsignedShort();
      }
    }
  }
  calcExtremes() {
    const start0 = this.markers[0];
    const start1 = this.markers[1];
    const end0 = this.markers[this.markers.length - 2];
    const end1 = this.markers[this.markers.length - 1];
    this.startMarker = [start0[0] + start0[0] - start1[0], start0[1] - start1[1] + start0[1]];
    this.endMarker = [end0[0] - end1[0] + end0[0], end0[1] - end1[1] + end0[1]];
  }
  getMarker(index) {
    if (index < 0) {
      return this.startMarker;
    }
    if (index >= this.markers.length) {
      return this.endMarker;
    }
    return this.markers[index];
  }
  init() {
    if (!this.markers) {
      this.markers = [
        [0, 0],
        [4096, 4096]
      ];
    }
    if (this.markers.length < 2) {
      throw new Error("Curve operation requires at least two markers");
    }
    if (this.interpMode === 2) {
      this.calcExtremes();
    }
    this.fillTable();
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const input = this.getMonochromeInput(textureGenerator, 0, line);
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        let value = input[pixel] / 16 | 0;
        if (value < 0) {
          value = 0;
        }
        if (value > 256) {
          value = 256;
        }
        output[pixel] = this.table[value];
      }
    }
    return output;
  }
  fillTable() {
    switch (this.interpMode) {
      case 2:
        for (let index = 0; index < 257; index++) {
          const indexTimes16 = index * 16;
          let markIndex;
          for (markIndex = 1; markIndex < this.markers.length - 1; markIndex++) {
            if (this.markers[markIndex][0] > indexTimes16) {
              break;
            }
          }
          const markP = this.markers[markIndex - 1];
          const markN = this.markers[markIndex];
          const i_17_ = this.getMarker(markIndex - 2)[1];
          const i_18_ = markP[1];
          const i_19_ = markN[1];
          const i_20_ = this.getMarker(markIndex + 1)[1];
          const interpIn = (indexTimes16 - markP[0]) * 4096 / (markN[0] - markP[0]) | 0;
          const xSq = interpIn * interpIn / 4096 | 0;
          const i_23_ = i_18_ - i_17_ + (i_20_ - i_19_);
          const i_24_ = i_17_ - i_18_ - i_23_;
          const i_25_ = i_19_ - i_17_;
          const i_26_ = i_18_;
          const i_27_ = xSq * (interpIn * i_23_ >> 12) >> 12;
          const i_28_ = xSq * i_24_ / 4096 | 0;
          const i_29_ = interpIn * i_25_ / 4096 | 0;
          let out = i_29_ + i_27_ + i_28_ + i_26_;
          if (out <= -32768) {
            out = -32767;
          }
          if (out >= 32768) {
            out = 32767;
          }
          this.table[index] = out;
        }
        break;
      case 1:
        for (let index = 0; index < 257; index++) {
          const indexTimes16 = index * 16;
          let markIndex;
          for (markIndex = 1; markIndex < this.markers.length - 1; markIndex++) {
            if (this.markers[markIndex][0] > indexTimes16) {
              break;
            }
          }
          const markP = this.markers[markIndex - 1];
          const markN = this.markers[markIndex];
          const interpIn = (indexTimes16 - markP[0]) * 4096 / (markN[0] - markP[0]) | 0;
          const nMul = (4096 - TextureGenerator.COSINE[(interpIn & 8187) / 32 | 0]) / 2 | 0;
          const pMul = 4096 - nMul;
          let out = (pMul * markP[1] + markN[1] * nMul) / 4096 | 0;
          if (out <= -32768) {
            out = -32767;
          }
          if (out >= 32768) {
            out = 32767;
          }
          this.table[index] = out;
        }
        break;
      case 0:
        for (let index = 0; index < 257; index++) {
          const indexTimes16 = index * 16;
          let markIndex;
          for (markIndex = 1; markIndex < this.markers.length - 1; markIndex++) {
            if (this.markers[markIndex][0] > indexTimes16) {
              break;
            }
          }
          const markP = this.markers[markIndex - 1];
          const markN = this.markers[markIndex];
          const nMul = (indexTimes16 - markP[0]) * 4096 / (markN[0] - markP[0]) | 0;
          const pMul = 4096 - nMul;
          let out = (pMul * markP[1] + markN[1] * nMul) / 4096 | 0;
          if (out <= -32768) {
            out = -32767;
          }
          if (out >= 32768) {
            out = 32767;
          }
          this.table[index] = out;
        }
        break;
    }
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/DiagonalGradientOperation.ts
var DiagonalGradientOperation = class extends TextureOperation {
  constructor() {
    super(0, true);
    this.interpolationMode = 0;
    this.steepness = 1;
    this.mixMode = 0;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.mixMode = buffer.readUnsignedByte();
    } else if (field === 1) {
      this.interpolationMode = buffer.readUnsignedByte();
    } else if (field === 3) {
      this.steepness = buffer.readUnsignedByte();
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const inParam = textureGenerator.verticalGradient[line];
      const nParam = inParam - 2048 >> 1;
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        const inValue = textureGenerator.horizontalGradient[pixel];
        const nValue = inValue - 2048 >> 1;
        let pytN;
        if (this.mixMode === 0) {
          pytN = (inValue - inParam) * this.steepness;
        } else {
          const sqNSum = nParam * nParam + nValue * nValue >> 12;
          pytN = 4096 * Math.sqrt(sqNSum / 4096) | 0;
          pytN = this.steepness * pytN * 3.141592653589793 | 0;
        }
        pytN -= pytN & ~4095;
        if (this.interpolationMode === 0) {
          pytN = TextureGenerator.SINE[pytN >> 4 & 255] + 4096 >> 1;
        } else if (this.interpolationMode === 2) {
          pytN -= 2048;
          if (pytN < 0) {
            pytN = -pytN;
          }
          pytN = 2048 - pytN << 1;
        }
        output[pixel] = pytN;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/EmbossOperation.ts
var EmbossOperation = class extends TextureOperation {
  constructor() {
    super(1, true);
    this.field0 = 4096;
    this.field1 = 3216;
    this.field2 = 3216;
    this.table = new Int32Array(3);
  }
  decode(field, buffer) {
    if (field === 0) {
      this.field0 = buffer.readUnsignedShort();
    } else if (field === 1) {
      this.field1 = buffer.readUnsignedShort();
    } else if (field === 2) {
      this.field2 = buffer.readUnsignedShort();
    }
  }
  init() {
    const d2 = Math.cos(Math.fround(this.field2 / 4096));
    this.table[0] = 4096 * (d2 * Math.sin(Math.fround(this.field1 / 4096)));
    this.table[1] = 4096 * (d2 * Math.cos(Math.fround(this.field1 / 4096)));
    this.table[2] = 4096 * Math.sin(Math.fround(this.field2 / 4096));
    const t0 = this.table[0] * this.table[0] >> 12;
    const t1 = this.table[1] * this.table[1] >> 12;
    const t2 = this.table[2] * this.table[2] >> 12;
    const scale = Math.sqrt(t0 + t1 + t2 >> 12) * 4096 | 0;
    if (scale !== 0) {
      this.table[0] = (this.table[0] << 12) / scale;
      this.table[1] = (this.table[1] << 12) / scale;
      this.table[2] = (this.table[2] << 12) / scale;
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const widthMult = this.field0 * textureGenerator.widthTimes32 >> 12;
      const prevLine = this.getMonochromeInput(
        textureGenerator,
        0,
        line - 1 & textureGenerator.heightMask
      );
      const currLine = this.getMonochromeInput(textureGenerator, 0, line);
      const nextLine = this.getMonochromeInput(
        textureGenerator,
        0,
        line + 1 & textureGenerator.heightMask
      );
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        const prevPixel = currLine[pixel - 1 & textureGenerator.widthMask];
        const nextPixel = currLine[pixel + 1 & textureGenerator.widthMask];
        const i_10_ = widthMult * (nextLine[pixel] - prevLine[pixel]) >> 12;
        const i_11_ = widthMult * (prevPixel - nextPixel) >> 12;
        let i_12_ = i_11_ >> 4;
        let i_13_ = i_10_ >> 4;
        if (i_12_ < 0) {
          i_12_ = -i_12_;
        }
        if (i_12_ > 255) {
          i_12_ = 255;
        }
        if (i_13_ < 0) {
          i_13_ = -i_13_;
        }
        if (i_13_ > 255) {
          i_13_ = 255;
        }
        const i_14_ = TextureGenerator.INVERSE_SQUARE_ROOT[i_12_ + ((i_13_ + 1) * i_13_ >> 1)] & 255;
        let v0 = i_14_ * i_11_ >> 8;
        let v1 = i_14_ * i_10_ >> 8;
        let v2 = i_14_ * 4096 >> 8;
        v0 = this.table[0] * v0 >> 12;
        v1 = this.table[1] * v1 >> 12;
        v2 = this.table[2] * v2 >> 12;
        output[pixel] = v0 + v1 + v2;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/GradientOperation.ts
var GradientOperation = class extends TextureOperation {
  constructor() {
    super(1, false);
    this.preset = 0;
    this.table = new Int32Array(257);
  }
  decode(field, buffer) {
    if (field === 0) {
      const preset = buffer.readUnsignedByte();
      if (preset === 0) {
        const count = buffer.readUnsignedByte();
        this.gradient = new Array(count);
        for (let i = 0; i < count; i++) {
          this.gradient[i] = new Int32Array(4);
          this.gradient[i][0] = buffer.readUnsignedShort();
          this.gradient[i][1] = buffer.readUnsignedByte() << 4;
          this.gradient[i][2] = buffer.readUnsignedByte() << 4;
          this.gradient[i][3] = buffer.readUnsignedByte() << 4;
        }
      } else {
        this.setGradientPreset(preset);
      }
    }
  }
  init() {
    if (!this.gradient) {
      this.setGradientPreset(1);
    }
    this.fillTable();
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const input = this.getMonochromeInput(textureGenerator, 0, line);
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        let value = input[pixel] >> 4;
        if (value < 0) {
          value = 0;
        }
        if (value > 256) {
          value = 256;
        }
        value = this.table[value];
        outputR[pixel] = (value & 16711680) >> 12;
        outputG[pixel] = (value & 65280) >> 4;
        outputB[pixel] = (value & 255) << 4;
      }
    }
    return output;
  }
  fillTable() {
    if (!this.gradient) {
      return;
    }
    const gradientCount = this.gradient.length;
    if (gradientCount <= 0) {
      return;
    }
    for (let i = 0; i < this.table.length; i++) {
      let gIdx = 0;
      const inT16 = i << 4;
      for (const grad of this.gradient) {
        if (grad[0] > inT16) {
          break;
        }
        gIdx++;
      }
      let r;
      let g;
      let b;
      if (gIdx < gradientCount) {
        const gradN = this.gradient[gIdx];
        if (gIdx > 0) {
          const gradP = this.gradient[gIdx - 1];
          const nMod = (inT16 - gradP[0] << 12) / (gradN[0] - gradP[0]) | 0;
          const pMod = 4096 - nMod;
          r = gradP[1] * pMod + gradN[1] * nMod >> 12;
          g = gradP[2] * pMod + gradN[2] * nMod >> 12;
          b = gradN[3] * nMod + gradP[3] * pMod >> 12;
        } else {
          r = gradN[1];
          g = gradN[2];
          b = gradN[3];
        }
      } else {
        const grad = this.gradient[gradientCount - 1];
        r = grad[1];
        g = grad[2];
        b = grad[3];
      }
      r >>= 4;
      g >>= 4;
      b >>= 4;
      if (r < 0) {
        r = 0;
      } else if (r > 255) {
        r = 255;
      }
      if (g < 0) {
        g = 0;
      } else if (g > 255) {
        g = 255;
      }
      if (b < 0) {
        b = 0;
      } else if (b > 255) {
        b = 255;
      }
      this.table[i] = r << 16 | g << 8 | b;
    }
  }
  setGradientPreset(preset) {
    this.preset = preset;
    switch (preset) {
      case 1:
        this.gradient = new Array(2);
        for (let i = 0; i < this.gradient.length; i++) {
          this.gradient[i] = new Int32Array(4);
        }
        this.gradient[0][0] = 0;
        this.gradient[0][1] = 0;
        this.gradient[0][2] = 0;
        this.gradient[0][3] = 0;
        this.gradient[1][0] = 4096;
        this.gradient[1][1] = 4096;
        this.gradient[1][2] = 4096;
        this.gradient[1][3] = 4096;
        break;
      case 2:
        this.gradient = new Array(8);
        for (let i = 0; i < this.gradient.length; i++) {
          this.gradient[i] = new Int32Array(4);
        }
        this.gradient[0][0] = 0;
        this.gradient[0][1] = 2650;
        this.gradient[0][2] = 2602;
        this.gradient[0][3] = 2361;
        this.gradient[1][0] = 2867;
        this.gradient[1][1] = 2313;
        this.gradient[1][2] = 1799;
        this.gradient[1][3] = 1558;
        this.gradient[2][0] = 3072;
        this.gradient[2][1] = 2618;
        this.gradient[2][2] = 1734;
        this.gradient[2][3] = 1413;
        this.gradient[3][0] = 3276;
        this.gradient[3][1] = 2296;
        this.gradient[3][2] = 1220;
        this.gradient[3][3] = 947;
        this.gradient[4][0] = 3481;
        this.gradient[4][1] = 2072;
        this.gradient[4][2] = 963;
        this.gradient[4][3] = 722;
        this.gradient[5][0] = 3686;
        this.gradient[5][1] = 2730;
        this.gradient[5][2] = 2152;
        this.gradient[5][3] = 1766;
        this.gradient[6][0] = 3891;
        this.gradient[6][1] = 2232;
        this.gradient[6][2] = 1060;
        this.gradient[6][3] = 915;
        this.gradient[7][0] = 4096;
        this.gradient[7][1] = 1686;
        this.gradient[7][2] = 1413;
        this.gradient[7][3] = 1140;
        break;
      case 3:
        this.gradient = new Array(7);
        for (let i = 0; i < this.gradient.length; i++) {
          this.gradient[i] = new Int32Array(4);
        }
        this.gradient[0][1] = 0;
        this.gradient[0][2] = 0;
        this.gradient[0][0] = 0;
        this.gradient[0][3] = 4096;
        this.gradient[1][1] = 0;
        this.gradient[1][0] = 663;
        this.gradient[1][3] = 4096;
        this.gradient[1][2] = 4096;
        this.gradient[2][2] = 4096;
        this.gradient[2][1] = 0;
        this.gradient[2][0] = 1363;
        this.gradient[2][3] = 0;
        this.gradient[3][3] = 0;
        this.gradient[3][2] = 4096;
        this.gradient[3][1] = 4096;
        this.gradient[3][0] = 2048;
        this.gradient[4][3] = 0;
        this.gradient[4][0] = 2727;
        this.gradient[4][2] = 0;
        this.gradient[4][1] = 4096;
        this.gradient[5][1] = 4096;
        this.gradient[5][0] = 3411;
        this.gradient[5][2] = 0;
        this.gradient[5][3] = 4096;
        this.gradient[6][3] = 4096;
        this.gradient[6][2] = 0;
        this.gradient[6][1] = 0;
        this.gradient[6][0] = 4096;
        break;
      case 4:
        this.gradient = new Array(6);
        for (let i = 0; i < this.gradient.length; i++) {
          this.gradient[i] = new Int32Array(4);
        }
        this.gradient[0][3] = 0;
        this.gradient[0][1] = 0;
        this.gradient[0][0] = 0;
        this.gradient[0][2] = 0;
        this.gradient[1][0] = 1843;
        this.gradient[1][3] = 1493;
        this.gradient[1][2] = 0;
        this.gradient[1][1] = 0;
        this.gradient[2][3] = 2939;
        this.gradient[2][0] = 2457;
        this.gradient[2][1] = 0;
        this.gradient[2][2] = 0;
        this.gradient[3][3] = 3565;
        this.gradient[3][0] = 2781;
        this.gradient[3][1] = 0;
        this.gradient[3][2] = 1124;
        this.gradient[4][3] = 4031;
        this.gradient[4][1] = 546;
        this.gradient[4][0] = 3481;
        this.gradient[4][2] = 3084;
        this.gradient[5][0] = 4096;
        this.gradient[5][2] = 4096;
        this.gradient[5][1] = 4096;
        this.gradient[5][3] = 4096;
        break;
      case 5:
        this.gradient = new Array(16);
        for (let i = 0; i < this.gradient.length; i++) {
          this.gradient[i] = new Int32Array(4);
        }
        this.gradient[0][2] = 192;
        this.gradient[0][0] = 0;
        this.gradient[0][1] = 80;
        this.gradient[0][3] = 321;
        this.gradient[1][1] = 321;
        this.gradient[1][0] = 155;
        this.gradient[1][3] = 562;
        this.gradient[1][2] = 449;
        this.gradient[2][1] = 578;
        this.gradient[2][0] = 389;
        this.gradient[2][3] = 803;
        this.gradient[2][2] = 690;
        this.gradient[3][2] = 995;
        this.gradient[3][0] = 671;
        this.gradient[3][3] = 1140;
        this.gradient[3][1] = 947;
        this.gradient[4][2] = 1397;
        this.gradient[4][1] = 1285;
        this.gradient[4][0] = 897;
        this.gradient[4][3] = 1509;
        this.gradient[5][2] = 1429;
        this.gradient[5][0] = 1175;
        this.gradient[5][3] = 1413;
        this.gradient[5][1] = 1525;
        this.gradient[6][3] = 1333;
        this.gradient[6][0] = 1368;
        this.gradient[6][1] = 1734;
        this.gradient[6][2] = 1461;
        this.gradient[7][0] = 1507;
        this.gradient[7][1] = 1413;
        this.gradient[7][3] = 1702;
        this.gradient[7][2] = 1525;
        this.gradient[8][1] = 1108;
        this.gradient[8][2] = 1590;
        this.gradient[8][3] = 2056;
        this.gradient[8][0] = 1736;
        this.gradient[9][1] = 1766;
        this.gradient[9][0] = 2088;
        this.gradient[9][3] = 2666;
        this.gradient[9][2] = 2056;
        this.gradient[10][2] = 2586;
        this.gradient[10][0] = 2355;
        this.gradient[10][1] = 2409;
        this.gradient[10][3] = 3276;
        this.gradient[11][1] = 3116;
        this.gradient[11][3] = 3228;
        this.gradient[11][2] = 3148;
        this.gradient[11][0] = 2691;
        this.gradient[12][2] = 3710;
        this.gradient[12][1] = 3806;
        this.gradient[12][3] = 3196;
        this.gradient[12][0] = 3031;
        this.gradient[13][1] = 3437;
        this.gradient[13][2] = 3421;
        this.gradient[13][3] = 3019;
        this.gradient[13][0] = 3522;
        this.gradient[14][1] = 3116;
        this.gradient[14][0] = 3727;
        this.gradient[14][2] = 3148;
        this.gradient[14][3] = 3228;
        this.gradient[15][1] = 2377;
        this.gradient[15][2] = 2505;
        this.gradient[15][3] = 2746;
        this.gradient[15][0] = 4096;
        break;
      case 6:
        this.gradient = new Array(4);
        for (let i = 0; i < this.gradient.length; i++) {
          this.gradient[i] = new Int32Array(4);
        }
        this.gradient[0][3] = 0;
        this.gradient[0][2] = 4096;
        this.gradient[0][0] = 2048;
        this.gradient[0][1] = 0;
        this.gradient[1][2] = 4096;
        this.gradient[1][1] = 4096;
        this.gradient[1][3] = 0;
        this.gradient[1][0] = 2867;
        this.gradient[2][1] = 4096;
        this.gradient[2][2] = 4096;
        this.gradient[2][3] = 0;
        this.gradient[2][0] = 3276;
        this.gradient[3][2] = 0;
        this.gradient[3][0] = 4096;
        this.gradient[3][3] = 0;
        this.gradient[3][1] = 4096;
        break;
      default:
        throw new Error(`Invalid gradient preset: ${preset}`);
    }
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/GrayScaleOperation.ts
var GrayScaleOperation = class extends TextureOperation {
  constructor() {
    super(1, true);
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const input = this.getColourInput(textureGenerator, 0, line);
      const inputR = input[0];
      const inputG = input[1];
      const inputB = input[2];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        output[pixel] = (inputR[pixel] + inputG[pixel] + inputB[pixel]) / 3;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/HerringboneOperation.ts
var HerringboneOperation = class extends TextureOperation {
  constructor() {
    super(0, true);
    this.scaleX = 1;
    this.scaleY = 1;
    this.ratio = 204;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.scaleX = buffer.readUnsignedByte();
    } else if (field === 1) {
      this.scaleY = buffer.readUnsignedByte();
    } else if (field === 2) {
      this.ratio = buffer.readUnsignedShort();
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      for (let x = 0; x < textureGenerator.width; x++) {
        const vGrad = textureGenerator.horizontalGradient[x];
        const hGrad = textureGenerator.verticalGradient[line];
        let local40 = this.scaleX * vGrad >> 12;
        const local51 = this.scaleY * hGrad >> 12;
        const local61 = this.scaleX * (vGrad % (4096 / this.scaleX | 0));
        const local71 = this.scaleY * (hGrad % (4096 / this.scaleY | 0));
        if (local71 < this.ratio) {
          for (local40 -= local51; local40 < 0; local40 += 4) {
          }
          while (local40 > 3) {
            local40 -= 4;
          }
          if (local40 != 1) {
            output[x] = 0;
            continue;
          }
          if (local61 < this.ratio) {
            output[x] = 0;
            continue;
          }
        }
        if (local61 < this.ratio) {
          let local131;
          for (local131 = local40 - local51; local131 < 0; local131 += 4) {
          }
          while (local131 > 3) {
            local131 -= 4;
          }
          if (local131 > 0) {
            output[x] = 0;
            continue;
          }
        }
        output[x] = 4096;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/HorizontalGradientOperation.ts
var HorizontalGradientOperation = class extends TextureOperation {
  constructor() {
    super(0, true);
  }
  getMonochromeOutput(textureGenerator, line) {
    return textureGenerator.horizontalGradient;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/HslOperation.ts
var HslOperation = class extends TextureOperation {
  constructor() {
    super(1, false);
    this.deltaHue = 0;
    this.deltaSaturation = 0;
    this.deltaLight = 0;
    this.hue = 0;
    this.saturation = 0;
    this.lightness = 0;
    this.r = 0;
    this.g = 0;
    this.b = 0;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.deltaHue = buffer.readSignedShort();
    } else if (field === 1) {
      this.deltaSaturation = (buffer.readByte() << 12) / 100 | 0;
    } else if (field === 2) {
      this.deltaLight = (buffer.readByte() << 12) / 100 | 0;
    }
  }
  setHsl(r, g, b) {
    const maxValue = Math.max(r, g, b);
    const minValue = Math.min(r, g, b);
    const delta = maxValue - minValue;
    this.lightness = (maxValue + minValue) / 2 | 0;
    if (delta > 0) {
      const invR = (maxValue - r << 12) / delta;
      const invG = (maxValue - g << 12) / delta;
      const invB = (maxValue - b << 12) / delta;
      if (r === maxValue) {
        this.hue = g === minValue ? invB + 20480 : 4096 - invG;
      } else if (g === maxValue) {
        this.hue = b === minValue ? invR + 4096 : 12288 - invB;
      } else {
        this.hue = minValue === r ? invG + 12288 : 20480 - invR;
      }
      this.hue = this.hue / 6 | 0;
    } else {
      this.hue = 0;
    }
    if (this.lightness > 0 && this.lightness < 4096) {
      this.saturation = (delta << 12) / (this.lightness > 2048 ? 8192 - this.lightness * 2 : this.lightness * 2);
    } else {
      this.saturation = 0;
    }
  }
  setRgb(hue, saturation, light) {
    const i = light > 2048 ? saturation + light - (saturation * light >> 12) : light * (4096 + saturation) >> 12;
    if (i > 0) {
      const j = light - i + light;
      const k = (i - j << 12) / i;
      hue *= 6;
      const l = hue >> 12;
      let j1 = i;
      let i1 = hue - (l << 12);
      j1 = j1 * k >> 12;
      j1 = i1 * j1 >> 12;
      const k1 = j + j1;
      const l1 = i - j1;
      if (l === 0) {
        this.r = i;
        this.g = k1;
        this.b = j;
      } else if (l === 1) {
        this.r = l1;
        this.g = i;
        this.b = j;
      } else if (l === 2) {
        this.r = j;
        this.g = i;
        this.b = k1;
      } else if (l === 3) {
        this.r = j;
        this.g = l1;
        this.b = i;
      } else if (l === 4) {
        this.r = k1;
        this.g = j;
        this.b = i;
      } else if (l === 5) {
        this.r = i;
        this.g = j;
        this.b = l1;
      }
    } else {
      this.r = this.g = this.b = light;
    }
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const input = this.getColourInput(textureGenerator, 0, line);
      const inputR = input[0];
      const inputG = input[1];
      const inputB = input[2];
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        this.setHsl(inputR[pixel], inputG[pixel], inputB[pixel]);
        this.hue += this.deltaHue;
        this.saturation += this.deltaSaturation;
        this.lightness += this.deltaLight;
        for (; this.hue < 0; this.hue += 4096) {
        }
        for (; this.hue > 4096; this.hue -= 4096) {
        }
        if (this.saturation < 0) {
          this.saturation = 0;
        }
        if (this.saturation > 4096) {
          this.saturation = 4096;
        }
        if (this.lightness < 0) {
          this.lightness = 0;
        }
        if (this.lightness > 4096) {
          this.lightness = 4096;
        }
        this.setRgb(this.hue, this.saturation, this.lightness);
        outputR[pixel] = this.r;
        outputG[pixel] = this.g;
        outputB[pixel] = this.b;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/InvertOperation.ts
var InvertOperation = class extends TextureOperation {
  constructor() {
    super(1, false);
  }
  decode(field, buffer) {
    if (field === 0) {
      this.isMonochrome = buffer.readUnsignedByte() === 1;
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const input = this.getMonochromeInput(textureGenerator, 0, line);
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        output[pixel] = 4096 - input[pixel];
      }
    }
    return output;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const input = this.getColourInput(textureGenerator, 0, line);
      const inputR = input[0];
      const inputG = input[1];
      const inputB = input[2];
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        outputR[pixel] = 4096 - inputR[pixel];
        outputG[pixel] = 4096 - inputG[pixel];
        outputB[pixel] = 4096 - inputB[pixel];
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/IrregularBricksOperation.ts
var import_java_random3 = __toESM(require_lib());

// {TSPS_CHECKOUT}/client/rs/util/ArrayUtils.ts
var ArrayUtils = class {
  static fill(array, start, length, value) {
    array.fill(value, start, start + length);
  }
  static fillRange(array, start, end, value) {
    array.fill(value, start, end);
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/IrregularBricksOperation.ts
var IrregularBricksOperation = class extends TextureOperation {
  constructor() {
    super(0, true);
    this.seed = 0;
    this.field1 = 1024;
    this.field2 = 2048;
    this.field3 = 409;
    this.field4 = 819;
    this.field5 = 1024;
    this.field6 = 0;
    this.field7 = 1024;
    this.field8 = 1024;
    this.anInt79 = 0;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.seed = buffer.readUnsignedByte();
    } else if (field === 1) {
      this.field1 = buffer.readUnsignedShort();
    } else if (field === 2) {
      this.field2 = buffer.readUnsignedShort();
    } else if (field === 3) {
      this.field3 = buffer.readUnsignedShort();
    } else if (field === 4) {
      this.field4 = buffer.readUnsignedShort();
    } else if (field === 5) {
      this.field5 = buffer.readUnsignedShort();
    } else if (field === 6) {
      this.field6 = buffer.readUnsignedByte();
    } else if (field === 7) {
      this.field7 = buffer.readUnsignedShort();
    } else if (field === 8) {
      this.field8 = buffer.readUnsignedShort();
    }
  }
  method69(textureGenerator, arg0, arg1, arg2, arg3, random, pixels) {
    const local20 = this.field8 > 0 ? 4096 - nextIntJagex(random, this.field8) : 4096;
    const local28 = this.anInt79 * this.field7 >> 12;
    const local44 = this.anInt79 - (local28 > 0 ? nextIntJagex(random, local28) : 0);
    if (textureGenerator.width <= arg2) {
      arg2 -= textureGenerator.width;
    }
    if (local44 > 0) {
      if (arg0 <= 0 || arg1 <= 0) {
        return;
      }
      const local67 = arg1 / 2 | 0;
      const local71 = arg0 / 2 | 0;
      const local82 = local67 >= local44 ? local44 : local67;
      const local93 = local44 > local71 ? local71 : local44;
      const local97 = arg2 + local82;
      const local104 = arg1 - local82 * 2;
      for (let local106 = 0; local106 < arg0; local106++) {
        const local116 = pixels[local106 + arg3];
        if (local93 <= local106) {
          const local260 = arg0 - local106 - 1;
          if (local93 <= local260) {
            for (let local403 = 0; local403 < local82; local403++) {
              local116[textureGenerator.widthMask & arg2 + local403] = local116[textureGenerator.widthMask & arg1 + arg2 - local403 - 1] = local20 * local403 / local82 | 0;
            }
            if (local97 + local104 <= textureGenerator.width) {
              ArrayUtils.fill(local116, local97, local104, local20);
            } else {
              const local461 = textureGenerator.width - local97;
              ArrayUtils.fill(local116, local97, local461, local20);
              ArrayUtils.fill(local116, 0, local104 - local461, local20);
            }
          } else {
            const local274 = local260 * local20 / local93 | 0;
            if (this.field6 === 0) {
              for (let local327 = 0; local327 < local82; local327++) {
                const local336 = local327 * local20 / local82 | 0;
                local116[textureGenerator.widthMask & arg2 + local327] = local116[arg1 + arg2 - local327 - 1 & textureGenerator.widthMask] = local274 * local336 >> 12;
              }
            } else {
              for (let local280 = 0; local280 < local82; local280++) {
                const local289 = local20 * local280 / local82 | 0;
                local116[textureGenerator.widthMask & local280 + arg2] = local116[textureGenerator.widthMask & arg2 + arg1 - local280 - 1] = local274 > local289 ? local289 : local274;
              }
            }
            if (textureGenerator.width < local104 + local97) {
              const local379 = textureGenerator.width - local97;
              ArrayUtils.fill(local116, local97, local379, local274);
              ArrayUtils.fill(local116, 0, local104 - local379, local274);
            } else {
              ArrayUtils.fill(local116, local97, local104, local274);
            }
          }
        } else {
          const local130 = local106 * local20 / local93 | 0;
          if (this.field6 === 0) {
            for (let local184 = 0; local184 < local82; local184++) {
              const local193 = local20 * local184 / local82 | 0;
              local116[textureGenerator.widthMask & local184 + arg2] = local116[arg2 + arg1 - local184 - 1 & textureGenerator.widthMask] = local193 * local130 >> 12;
            }
          } else {
            for (let local138 = 0; local138 < local82; local138++) {
              const local151 = local20 * local138 / local82 | 0;
              local116[local138 + arg2 & textureGenerator.widthMask] = local116[arg1 + arg2 - local138 - 1 & textureGenerator.widthMask] = local130 <= local151 ? local130 : local151;
            }
          }
          if (local97 + local104 > textureGenerator.width) {
            const local231 = textureGenerator.width - local97;
            ArrayUtils.fill(local116, local97, local231, local130);
            ArrayUtils.fill(local116, 0, local104 - local231, local130);
          } else {
            ArrayUtils.fill(local116, local97, local104, local130);
          }
        }
      }
    } else if (textureGenerator.width >= arg1 + arg2) {
      for (let i = 0; i < arg0; i++) {
        ArrayUtils.fill(pixels[i + arg3], arg2, arg1, local20);
      }
    } else {
      const local507 = textureGenerator.width - arg2;
      for (let i = 0; i < arg0; i++) {
        const local518 = pixels[i + arg3];
        ArrayUtils.fill(local518, arg2, local507, local20);
        ArrayUtils.fill(local518, 0, arg1 - local507, local20);
      }
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (!this.monochromeImageCache.dirty) {
      return output;
    }
    let local23 = 0;
    const pixels = this.monochromeImageCache.getAll();
    let local30 = 0;
    let local32 = 0;
    let local34 = 0;
    let local36 = 0;
    let local38 = true;
    let local40 = 0;
    let local42 = true;
    const local49 = this.field1 * textureGenerator.width >> 12;
    let local51 = 0;
    const local58 = this.field3 * textureGenerator.height >> 12;
    const local65 = textureGenerator.width * this.field2 >> 12;
    const local72 = this.field4 * textureGenerator.height >> 12;
    if (local72 <= 1) {
      return pixels[line];
    }
    this.anInt79 = textureGenerator.width / 8 * this.field5 >> 12;
    const local99 = textureGenerator.width / local49 + 1 | 0;
    const random = new import_java_random3.default(this.seed);
    let local110 = new Array(local99);
    let local114 = new Array(local99);
    for (let i = 0; i < local99; i++) {
      local110[i] = new Int32Array(3);
      local114[i] = new Int32Array(3);
    }
    while (true) {
      while (true) {
        let local125 = local49 + nextIntJagex(random, local65 - local49);
        let local135 = nextIntJagex(random, local72 - local58) + local58;
        let local139 = local32 + local125;
        if (local139 > textureGenerator.width) {
          local125 = textureGenerator.width - local32;
          local139 = textureGenerator.width;
        }
        let local153;
        if (local38) {
          local153 = 0;
        } else {
          let local157 = local36;
          const local161 = local114[local36];
          local153 = local161[2];
          let local167 = 0;
          let local171 = local23 + local139;
          if (local171 < 0) {
            local171 += textureGenerator.width;
          }
          if (textureGenerator.width < local171) {
            local171 -= textureGenerator.width;
          }
          while (true) {
            const local196 = local114[local157];
            if (local196[0] <= local171 && local171 <= local196[1]) {
              if (local157 !== local36) {
                let local238 = local32 + local23;
                if (local238 < 0) {
                  local238 += textureGenerator.width;
                }
                if (local238 > textureGenerator.width) {
                  local238 -= textureGenerator.width;
                }
                for (let local258 = 1; local258 <= local167; local258++) {
                  const local269 = local114[(local258 + local36) % local40];
                  local153 = Math.max(local153, local269[2]);
                }
                for (let local280 = 0; local280 <= local167; local280++) {
                  const local295 = local114[(local280 + local36) % local40];
                  const local299 = local295[2];
                  if (local299 !== local153) {
                    const local306 = local295[1];
                    const local310 = local295[0];
                    let local320;
                    let local322;
                    if (local238 < local171) {
                      local320 = Math.max(local238, local310);
                      local322 = Math.min(local171, local306);
                    } else if (local310 == 0) {
                      local322 = Math.min(local171, local306);
                      local320 = 0;
                    } else {
                      local320 = Math.max(local238, local310);
                      local322 = textureGenerator.width;
                    }
                    this.method69(
                      textureGenerator,
                      local153 - local299,
                      local322 - local320,
                      local34 + local320,
                      local299,
                      random,
                      pixels
                    );
                  }
                }
              }
              local36 = local157;
              break;
            }
            local157++;
            if (local157 >= local40) {
              local157 = 0;
            }
            local167++;
          }
        }
        if (textureGenerator.height < local135 + local153) {
          local135 = textureGenerator.height - local153;
        } else {
          local42 = false;
        }
        if (local139 === textureGenerator.width) {
          this.method69(
            textureGenerator,
            local135,
            local125,
            local32 + local30,
            local153,
            random,
            pixels
          );
          if (local42) {
            return output;
          }
          local38 = false;
          const local440 = local51 + 1;
          const local442 = local110[local51];
          local42 = true;
          local442[1] = local139;
          local34 = local30;
          local40 = local440;
          local442[0] = local32;
          local442[2] = local135 + local153;
          local30 = nextIntJagex(random, textureGenerator.width);
          const local469 = local114;
          local36 = 0;
          local23 = local30 - local34;
          local114 = local110;
          let local480 = local23;
          local110 = local469;
          if (local23 < 0) {
            local480 = local23 + textureGenerator.width;
          }
          local51 = 0;
          if (textureGenerator.width < local480) {
            local480 -= textureGenerator.width;
          }
          while (true) {
            const local506 = local114[local36];
            if (local480 >= local506[0] && local506[1] >= local480) {
              local32 = 0;
              break;
            }
            local36++;
            if (local40 <= local36) {
              local36 = 0;
            }
          }
        } else {
          const local388 = local110[local51++];
          local388[1] = local139;
          local388[2] = local135 + local153;
          local388[0] = local32;
          this.method69(
            textureGenerator,
            local135,
            local125,
            local30 + local32,
            local153,
            random,
            pixels
          );
          local32 = local139;
        }
      }
    }
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/KaleidoscopeOperation.ts
var KaleidoscopeOperation = class _KaleidoscopeOperation extends TextureOperation {
  static {
    this.x0 = 0;
  }
  static {
    this.y0 = 0;
  }
  constructor() {
    super(1, false);
  }
  decode(field, buffer) {
    if (field === 0) {
      this.isMonochrome = buffer.readUnsignedByte() === 1;
    }
  }
  calcPos(textureGenerator, x, y) {
    const hGrad = textureGenerator.horizontalGradient[y];
    const vGrad = textureGenerator.verticalGradient[x];
    const angle = Math.fround(Math.atan2(hGrad - 2048, vGrad - 2048));
    if (angle >= -3.141592653589793 && angle <= -2.356194490192345) {
      _KaleidoscopeOperation.x0 = x;
      _KaleidoscopeOperation.y0 = y;
    } else if (angle <= -1.5707963267948966 && angle >= -2.356194490192345) {
      _KaleidoscopeOperation.y0 = x;
      _KaleidoscopeOperation.x0 = y;
    } else if (angle <= -0.7853981633974483 && angle >= -1.5707963267948966) {
      _KaleidoscopeOperation.x0 = textureGenerator.width - y;
      _KaleidoscopeOperation.y0 = x;
    } else if (angle <= 0 && angle >= -0.7853981633974483) {
      _KaleidoscopeOperation.y0 = textureGenerator.height - y;
      _KaleidoscopeOperation.x0 = x;
    } else if (angle >= 0 && angle <= 0.7853981633974483) {
      _KaleidoscopeOperation.x0 = textureGenerator.width - x;
      _KaleidoscopeOperation.y0 = textureGenerator.height - y;
    } else if (angle >= 0.7853981633974483 && angle <= 1.5707963267948966) {
      _KaleidoscopeOperation.x0 = textureGenerator.width - y;
      _KaleidoscopeOperation.y0 = textureGenerator.height - x;
    } else if (angle >= 1.5707963267948966 && angle <= 2.356194490192345) {
      _KaleidoscopeOperation.x0 = y;
      _KaleidoscopeOperation.y0 = textureGenerator.height - x;
    } else if (angle >= 2.356194490192345 && angle <= 3.141592653589793) {
      _KaleidoscopeOperation.y0 = y;
      _KaleidoscopeOperation.x0 = textureGenerator.width - x;
    }
    _KaleidoscopeOperation.x0 &= textureGenerator.widthMask;
    _KaleidoscopeOperation.y0 &= textureGenerator.heightMask;
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        this.calcPos(textureGenerator, pixel, line);
        const input = this.getMonochromeInput(
          textureGenerator,
          0,
          _KaleidoscopeOperation.y0
        );
        output[pixel] = input[_KaleidoscopeOperation.x0];
      }
    }
    return output;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        this.calcPos(textureGenerator, pixel, line);
        const input = this.getColourInput(textureGenerator, 0, _KaleidoscopeOperation.y0);
        outputR[pixel] = input[0][_KaleidoscopeOperation.x0];
        outputG[pixel] = input[1][_KaleidoscopeOperation.x0];
        outputB[pixel] = input[2][_KaleidoscopeOperation.x0];
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/LineNoiseOperation.ts
var import_java_random4 = __toESM(require_lib());
var LineNoiseOperation = class extends TextureOperation {
  constructor() {
    super(0, true);
    this.seed = 0;
    this.count = 2e3;
    this.length = 16;
    this.minAngle = 0;
    this.maxAngle = 4096;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.seed = buffer.readUnsignedByte();
    } else if (field === 1) {
      this.count = buffer.readUnsignedShort();
    } else if (field === 2) {
      this.length = buffer.readUnsignedByte();
    } else if (field === 3) {
      this.minAngle = buffer.readUnsignedShort();
    } else if (field === 4) {
      this.maxAngle = buffer.readUnsignedShort();
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const midAngle = this.maxAngle >> 1;
      const pixels = this.monochromeImageCache.getAll();
      const random = new import_java_random4.default(this.seed);
      for (let i = 0; i < this.count; i++) {
        let angle = this.maxAngle > 0 ? this.minAngle - midAngle + nextIntJagex(random, this.maxAngle) : this.minAngle;
        angle = angle >> 4 & 255;
        let x0 = nextIntJagex(random, textureGenerator.width);
        let y0 = nextIntJagex(random, textureGenerator.height);
        let x1 = (TextureGenerator.COSINE[angle] * this.length >> 12) + x0;
        let y1 = (TextureGenerator.SINE[angle] * this.length >> 12) + y0;
        let deltaX = x1 - x0;
        let deltaY = y1 - y0;
        if (deltaX !== 0 || deltaY !== 0) {
          if (deltaX < 0) {
            deltaX = -deltaX;
          }
          if (deltaY < 0) {
            deltaY = -deltaY;
          }
          const flag = deltaX < deltaY;
          if (flag) {
            const tempX0 = x0;
            const tempX1 = x1;
            x0 = y0;
            y0 = tempX0;
            x1 = y1;
            y1 = tempX1;
          }
          if (x0 > x1) {
            const tempX0 = x0;
            const tempY0 = y0;
            x0 = x1;
            y0 = y1;
            x1 = tempX0;
            y1 = tempY0;
          }
          const deltaX0 = x1 - x0;
          let deltaY0 = y1 - y0;
          let l2 = y0;
          if (deltaY0 < 0) {
            deltaY0 = -deltaY0;
          }
          let i4 = -deltaX0 / 2 | 0;
          const j4 = 2048 / deltaX0 | 0;
          const k4 = 1024 - (nextIntJagex(random, 4096) >> 2);
          const byte0 = y1 <= y0 ? -1 : 1;
          for (let x = x0; x < x1; x++) {
            i4 += deltaY0;
            const value = j4 * (x - x0) + (1024 + k4);
            const line2 = l2 & textureGenerator.heightMask;
            if (i4 > 0) {
              l2 = byte0 + l2;
              i4 = i4 - deltaX0;
            }
            const pixel = x & textureGenerator.widthMask;
            if (!flag) {
              pixels[pixel][line2] = value;
            } else {
              pixels[line2][pixel] = value;
            }
          }
        }
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/MandelbrotOperation.ts
var MandelbrotOperation = class extends TextureOperation {
  constructor() {
    super(0, true);
    this.field0 = 1365;
    this.field1 = 20;
    this.field2 = 0;
    this.field3 = 0;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.field0 = buffer.readUnsignedShort();
    } else if (field === 1) {
      this.field1 = buffer.readUnsignedShort();
    } else if (field === 2) {
      this.field2 = buffer.readUnsignedShort();
    } else if (field === 3) {
      this.field3 = buffer.readUnsignedShort();
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      for (let x = 0; x < textureGenerator.width; x++) {
        const local42 = this.field2 + (textureGenerator.horizontalGradient[x] << 12) / this.field0 | 0;
        const local54 = this.field3 + (textureGenerator.verticalGradient[line] << 12) / this.field0 | 0;
        let local58 = local54;
        let local60 = local42;
        let local64 = 0;
        let local70 = local42 * local42 >> 12;
        let local76 = local54 * local54 >> 12;
        while (local70 + local76 < 16384 && local64 < this.field1) {
          local64++;
          local58 = local54 + (local58 * local60 >> 12) * 2;
          local60 = local42 + local70 - local76;
          local76 = local58 * local58 >> 12;
          local70 = local60 * local60 >> 12;
        }
        output[x] = local64 >= this.field1 - 1 ? 0 : (local64 << 12) / this.field1 | 0;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/MirrorOperation.ts
var MirrorOperation = class extends TextureOperation {
  constructor() {
    super(1, false);
    this.invertHorizontal = true;
    this.invertVertical = true;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.invertHorizontal = buffer.readUnsignedByte() === 1;
    } else if (field === 1) {
      this.invertVertical = buffer.readUnsignedByte() === 1;
    } else if (field === 2) {
      this.isMonochrome = buffer.readUnsignedByte() === 1;
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const input = this.getMonochromeInput(
        textureGenerator,
        0,
        this.invertVertical ? textureGenerator.heightMask - line : line
      );
      if (this.invertHorizontal) {
        for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
          output[pixel] = input[textureGenerator.widthMask - pixel];
        }
      } else {
        output.set(input);
      }
    }
    return output;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const input = this.getColourInput(
        textureGenerator,
        0,
        this.invertVertical ? textureGenerator.heightMask - line : line
      );
      const inputR = input[0];
      const inputG = input[1];
      const inputB = input[2];
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      if (this.invertHorizontal) {
        for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
          outputR[pixel] = inputR[textureGenerator.widthMask - pixel];
          outputG[pixel] = inputG[textureGenerator.widthMask - pixel];
          outputB[pixel] = inputB[textureGenerator.widthMask - pixel];
        }
      } else {
        for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
          outputR[pixel] = inputR[pixel];
          outputG[pixel] = inputG[pixel];
          outputB[pixel] = inputB[pixel];
        }
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/MixerOperation.ts
var MixerOperation = class extends TextureOperation {
  constructor() {
    super(3, false);
  }
  decode(field, buffer) {
    if (field === 0) {
      this.isMonochrome = buffer.readUnsignedByte() === 1;
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const inputA = this.getMonochromeInput(textureGenerator, 0, line);
      const inputB = this.getMonochromeInput(textureGenerator, 1, line);
      const inputC = this.getMonochromeInput(textureGenerator, 2, line);
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        const aWeight = inputC[pixel];
        if (aWeight === 4096) {
          output[pixel] = inputA[pixel];
        } else if (aWeight === 0) {
          output[pixel] = inputB[pixel];
        } else {
          output[pixel] = (4096 - aWeight) * inputB[pixel] + aWeight * inputA[pixel] >> 12;
        }
      }
    }
    return output;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const inputC = this.getMonochromeInput(textureGenerator, 2, line);
      const inputA = this.getColourInput(textureGenerator, 0, line);
      const inputB = this.getColourInput(textureGenerator, 1, line);
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      const inputAR = inputA[0];
      const inputAG = inputA[1];
      const inputAB = inputA[2];
      const inputBR = inputB[0];
      const inputBG = inputB[1];
      const inputBB = inputB[2];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        const aWeight = inputC[pixel];
        if (aWeight === 4096) {
          outputR[pixel] = inputAR[pixel];
          outputG[pixel] = inputAG[pixel];
          outputB[pixel] = inputAB[pixel];
        } else if (aWeight === 0) {
          outputR[pixel] = inputBR[pixel];
          outputG[pixel] = inputBG[pixel];
          outputB[pixel] = inputBB[pixel];
        } else {
          const bWeight = 4096 - aWeight;
          outputR[pixel] = aWeight * inputAR[pixel] + bWeight * inputBR[pixel] >> 12;
          outputG[pixel] = aWeight * inputAG[pixel] + bWeight * inputBG[pixel] >> 12;
          outputB[pixel] = aWeight * inputAB[pixel] + bWeight * inputBB[pixel] >> 12;
        }
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/MonochromeEdgeDetectorOperation.ts
var MonochromeEdgeDetectorOperation = class extends TextureOperation {
  constructor() {
    super(1, true);
    this.multiplier = 4096;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.multiplier = buffer.readUnsignedShort();
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const prevInput = this.getMonochromeInput(
        textureGenerator,
        0,
        line - 1 & textureGenerator.heightMask
      );
      const input = this.getMonochromeInput(textureGenerator, 0, line);
      const nextInput = this.getMonochromeInput(
        textureGenerator,
        0,
        line + 1 & textureGenerator.heightMask
      );
      for (let x = 0; x < textureGenerator.width; x++) {
        const dy = this.multiplier * (nextInput[x] - prevInput[x]);
        const dx = this.multiplier * (input[x + 1 & textureGenerator.widthMask] - input[x - 1 & textureGenerator.widthMask]);
        const dx0 = dx >> 12;
        const dy0 = dy >> 12;
        const dySquared = dy0 * dy0 >> 12;
        const dxSquared = dx0 * dx0 >> 12;
        const local117 = Math.sqrt((dySquared + dxSquared + 4096) / 4096) * 4096 | 0;
        const local128 = local117 == 0 ? 0 : 16777216 / local117 | 0;
        output[x] = 4096 - local128;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/Operation37.ts
var Operation37 = class extends TextureOperation {
  constructor() {
    super(0, true);
    this.field0 = 2048;
    this.field1 = 0;
    this.field2 = 0;
    this.field3 = 2048;
    this.field4 = 12288;
    this.field5 = 4096;
    this.field6 = 8192;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.field0 = buffer.readUnsignedShort();
    } else if (field === 1) {
      this.field1 = buffer.readUnsignedShort();
    } else if (field === 2) {
      this.field2 = buffer.readUnsignedShort();
    } else if (field === 3) {
      this.field3 = buffer.readUnsignedShort();
    } else if (field === 4) {
      this.field4 = buffer.readUnsignedShort();
    } else if (field === 5) {
      this.field5 = buffer.readUnsignedShort();
    } else if (field === 6) {
      this.field6 = buffer.readUnsignedShort();
    }
  }
  checkDiagonalA(x, y) {
    const local9 = (y - x) * this.field4 >> 12;
    let local24 = TextureGenerator.COSINE[local9 * 255 >> 12 & 255];
    local24 = (local24 << 12) / this.field4 | 0;
    local24 = (local24 << 12) / this.field6 | 0;
    local24 = this.field5 * local24 >> 12;
    return local24 > x + y && -local24 < x + y;
  }
  checkDiagonalB(x, y) {
    const local13 = (y + x) * this.field4 >> 12;
    let local23 = TextureGenerator.COSINE[local13 * 255 >> 12 & 255];
    local23 = (local23 << 12) / this.field4 | 0;
    local23 = (local23 << 12) / this.field6 | 0;
    local23 = local23 * this.field5 >> 12;
    return local23 > y - x && y - x > -local23;
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const local22 = textureGenerator.verticalGradient[line] - 2048;
      for (let x = 0; x < textureGenerator.width; x++) {
        const local33 = textureGenerator.horizontalGradient[x] - 2048;
        let local38 = local33 + this.field0;
        local38 = local38 >= -2048 ? local38 : local38 + 4096;
        let local53 = local22 + this.field1;
        local38 = local38 <= 2048 ? local38 : local38 - 4096;
        local53 = local53 >= -2048 ? local53 : local53 + 4096;
        local53 = local53 <= 2048 ? local53 : local53 - 4096;
        let local87 = local33 + this.field2;
        let local92 = local22 + this.field3;
        local87 = local87 >= -2048 ? local87 : local87 + 4096;
        local87 = local87 <= 2048 ? local87 : local87 - 4096;
        local92 = local92 >= -2048 ? local92 : local92 + 4096;
        local92 = local92 <= 2048 ? local92 : local92 - 4096;
        output[x] = this.checkDiagonalA(local38, local53) || this.checkDiagonalB(local87, local92) ? 4096 : 0;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/PerlinNoiseOperation.ts
var PerlinNoiseOperation = class _PerlinNoiseOperation extends TextureOperation {
  constructor() {
    super(0, true);
    this.field0 = true;
    this.field1 = 4;
    this.field2 = 1638;
    this.seed = 0;
    this.field5 = 4;
    this.field6 = 4;
    this.permutations = new Int8Array(512);
  }
  static {
    this.invertTable = [
      [1, 1],
      [-1, 1],
      [1, -1],
      [-1, -1]
    ];
  }
  static {
    this.noise = new Int32Array(4096);
  }
  static initNoise() {
    for (let i = 0; i < 4096; i++) {
      _PerlinNoiseOperation.noise[i] = _PerlinNoiseOperation.calcNoise(i);
    }
  }
  static addInvert(x, y, invert) {
    return x * invert[0] + y * invert[1];
  }
  static smoothStep(n) {
    const i = (n * n >> 12) * n >> 12;
    const j = 6 * n - 61440;
    const k = 40960 + (j * n >> 12);
    return k * i >> 12;
  }
  static calcNoise(n) {
    const i_9_ = (n * n >> 12) * n >> 12;
    const i_10_ = n * 6 - 61440;
    const i_11_ = 40960 + (n * i_10_ >> 12);
    return i_9_ * i_11_ >> 12;
  }
  static lerp(start, end, amount) {
    return start * (4096 - amount) + end * amount >> 12;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.field0 = buffer.readUnsignedByte() === 1;
    } else if (field === 1) {
      this.field1 = buffer.readUnsignedByte();
    } else if (field === 2) {
      this.field2 = buffer.readSignedShort();
      if (this.field2 < 0) {
        this.noiseInput0 = new Int16Array(this.field1);
        for (let i = 0; i < this.field1; i++) {
          this.noiseInput0[i] = buffer.readSignedShort();
        }
      }
    } else if (field === 3) {
      this.field5 = this.field6 = buffer.readUnsignedByte();
    } else if (field === 4) {
      this.seed = buffer.readUnsignedByte();
    } else if (field === 5) {
      this.field5 = buffer.readUnsignedByte();
    } else if (field === 6) {
      this.field6 = buffer.readUnsignedByte();
    }
  }
  init() {
    this.initTable();
    this.initNoiseInput();
    for (let i = this.field1 - 1; i >= 1; i--) {
      const v = this.noiseInput0[i];
      if (v > 8 || v < -8) {
        break;
      }
      this.field1--;
    }
  }
  initTable() {
    this.permutations = TextureGenerator.initPermutations(this.seed);
  }
  initNoiseInput() {
    if (this.field2 <= 0) {
      if (this.noiseInput0 && this.noiseInput0.length === this.field1) {
        this.noiseInput1 = new Int16Array(this.field1);
        for (let i = 0; i < this.field1; i++) {
          this.noiseInput1[i] = Math.pow(2, i);
        }
      }
    } else {
      this.noiseInput0 = new Int16Array(this.field1);
      this.noiseInput1 = new Int16Array(this.field1);
      for (let i = 0; i < this.field1; i++) {
        this.noiseInput0[i] = Math.pow(Math.fround(this.field2 / 4096), i) * 4096;
        this.noiseInput1[i] = Math.pow(2, i);
      }
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      this.noise0(textureGenerator, line, output);
    }
    return output;
  }
  noise0(textureGenerator, line, output) {
    const vGrad = this.field6 * textureGenerator.verticalGradient[line];
    if (this.field1 === 1) {
      const nin0 = this.noiseInput0[0];
      const nin1 = this.noiseInput1[0] << 12;
      const n1f5 = nin1 * this.field5 >> 12;
      const n1f6 = nin1 * this.field6 >> 12;
      let noiseIndex = nin1 * vGrad >> 12;
      const permIndex0 = noiseIndex >> 12;
      let permIndex1 = permIndex0 + 1;
      if (n1f6 <= permIndex1) {
        permIndex1 = 0;
      }
      noiseIndex &= 4095;
      const noise = _PerlinNoiseOperation.noise[noiseIndex];
      const perm0 = this.permutations[permIndex0 & 255] & 255;
      const perm1 = this.permutations[permIndex1 & 255] & 255;
      if (this.field0) {
        for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
          const hGrad = this.field5 * textureGenerator.horizontalGradient[pixel];
          let v = this.noise1(
            nin1 * hGrad >> 12,
            n1f5,
            perm0,
            perm1,
            noiseIndex,
            noise
          );
          v = nin0 * v >> 12;
          output[pixel] = (v >> 1) + 2048;
        }
      } else {
        for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
          const hGrad = this.field5 * textureGenerator.horizontalGradient[pixel];
          const v = this.noise1(
            nin1 * hGrad >> 12,
            n1f5,
            perm0,
            perm1,
            noiseIndex,
            noise
          );
          output[pixel] = v * nin0 >> 12;
        }
      }
    } else {
      let i_45_ = this.noiseInput0[0];
      if (i_45_ > 8 || i_45_ < -8) {
        const i_46_ = this.noiseInput1[0] << 12;
        let i_47_ = i_46_ * vGrad >> 12;
        const i_48_ = i_46_ * this.field5 >> 12;
        const i_49_ = i_46_ * this.field6 >> 12;
        const i_50_ = i_47_ >> 12;
        let i_51_ = i_50_ + 1;
        i_47_ &= 4095;
        if (i_49_ <= i_51_) {
          i_51_ = 0;
        }
        const i_52_ = this.permutations[i_50_ & 255] & 255;
        const i_53_ = _PerlinNoiseOperation.noise[i_47_];
        const i_54_ = this.permutations[i_51_ & 255] & 255;
        for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
          const i_56_ = this.field5 * textureGenerator.horizontalGradient[pixel];
          const i_57_ = this.noise1(
            i_56_ * i_46_ >> 12,
            i_48_,
            i_52_,
            i_54_,
            i_47_,
            i_53_
          );
          output[pixel] = i_45_ * i_57_ >> 12;
        }
      }
      for (let i_58_ = 1; i_58_ < this.field1; i_58_++) {
        i_45_ = this.noiseInput0[i_58_];
        if (i_45_ > 8 || i_45_ < -8) {
          const i_59_ = this.noiseInput1[i_58_] << 12;
          const i_60_ = this.field6 * i_59_ >> 12;
          const i_61_ = this.field5 * i_59_ >> 12;
          let i_62_ = vGrad * i_59_ >> 12;
          const i_63_ = i_62_ >> 12;
          let i_64_ = i_63_ + 1;
          i_62_ &= 4095;
          if (i_60_ <= i_64_) {
            i_64_ = 0;
          }
          const i_65_ = this.permutations[i_64_ & 255] & 255;
          const i_66_ = this.permutations[i_63_ & 255] & 255;
          const i_67_ = _PerlinNoiseOperation.noise[i_62_];
          if (this.field0 && this.field1 - 1 === i_58_) {
            for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
              const i_69_ = textureGenerator.horizontalGradient[pixel] * this.field5;
              let i_70_ = this.noise1(
                i_59_ * i_69_ >> 12,
                i_61_,
                i_66_,
                i_65_,
                i_62_,
                i_67_
              );
              i_70_ = output[pixel] + (i_70_ * i_45_ >> 12);
              output[pixel] = 2048 + (i_70_ >> 1);
            }
          } else {
            for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
              const i_72_ = textureGenerator.horizontalGradient[pixel] * this.field5;
              const i_73_ = this.noise1(
                i_59_ * i_72_ >> 12,
                i_61_,
                i_66_,
                i_65_,
                i_62_,
                i_67_
              );
              output[pixel] += i_45_ * i_73_ >> 12;
            }
          }
        }
      }
    }
  }
  noise1(hGrad, vGrad, perm0, perm1, noiseIndex, noise) {
    let i_8_ = hGrad >> 12;
    let i_9_ = i_8_ + 1;
    hGrad &= 4095;
    if (i_9_ >= vGrad) {
      i_9_ = 0;
    }
    i_8_ &= 255;
    let i_10_ = noiseIndex - 4096;
    let i_11_ = hGrad - 4096;
    i_9_ &= 255;
    let i_12_ = this.permutations[perm0 + i_8_] & 3;
    let i_13_ = _PerlinNoiseOperation.noise[hGrad];
    let i_14_;
    if (i_12_ > 1) {
      i_14_ = i_12_ == 2 ? -noiseIndex + hGrad : -noiseIndex + -hGrad;
    } else {
      i_14_ = i_12_ == 0 ? noiseIndex + hGrad : -hGrad + noiseIndex;
    }
    i_12_ = this.permutations[perm0 + i_9_] & 3;
    let i_15_;
    if (i_12_ <= 1) {
      i_15_ = i_12_ == 0 ? noiseIndex + i_11_ : noiseIndex - i_11_;
    } else {
      i_15_ = i_12_ == 2 ? i_11_ - noiseIndex : -i_11_ + -noiseIndex;
    }
    i_12_ = this.permutations[perm1 + i_8_] & 3;
    const i_16_ = (i_13_ * (i_15_ - i_14_) >> 12) + i_14_;
    if (i_12_ <= 1) {
      i_14_ = i_12_ != 0 ? i_10_ - hGrad : hGrad + i_10_;
    } else {
      i_14_ = i_12_ != 2 ? -i_10_ + -hGrad : hGrad - i_10_;
    }
    i_12_ = this.permutations[i_9_ + perm1] & 3;
    if (i_12_ <= 1) {
      i_15_ = i_12_ == 0 ? i_11_ + i_10_ : i_10_ - i_11_;
    } else {
      i_15_ = i_12_ == 2 ? -i_10_ + i_11_ : -i_10_ + -i_11_;
    }
    const i_17_ = i_14_ + (i_13_ * (i_15_ - i_14_) >> 12);
    return i_16_ + (noise * (i_17_ - i_16_) >> 12);
  }
  noise(x, y, verticalGradient, horizontalGradient) {
    let k = x & 4294963200;
    x -= k;
    let l = y & 4294963200;
    y -= l;
    const j1 = verticalGradient & 4294963200;
    const i1 = horizontalGradient & 4294963200;
    l >>= 12;
    let j = l + 1;
    l &= 255;
    k >>= 12;
    let i = k + 1;
    if (i1 >> 12 <= i) {
      i = 0;
    }
    k &= 255;
    i &= 255;
    if (j >= j1 >> 12) {
      j = 0;
    }
    const i2 = this.permutations[this.permutations[l] + i] % 4;
    const k1 = this.permutations[this.permutations[l] + k] % 4;
    j &= 255;
    const j2 = this.permutations[this.permutations[j] + i] % 4;
    const l1 = this.permutations[this.permutations[j] + k] % 4;
    const k2 = _PerlinNoiseOperation.addInvert(x, y, _PerlinNoiseOperation.invertTable[k1]);
    const l2 = _PerlinNoiseOperation.addInvert(
      x - 4096,
      y,
      _PerlinNoiseOperation.invertTable[i2]
    );
    const i3 = _PerlinNoiseOperation.addInvert(
      x,
      y - 4096,
      _PerlinNoiseOperation.invertTable[l1]
    );
    const j3 = _PerlinNoiseOperation.addInvert(
      x - 4096,
      y - 4096,
      _PerlinNoiseOperation.invertTable[j2]
    );
    const k3 = _PerlinNoiseOperation.smoothStep(x);
    const l3 = _PerlinNoiseOperation.smoothStep(y);
    const i4 = _PerlinNoiseOperation.lerp(k2, l2, k3);
    const j4 = _PerlinNoiseOperation.lerp(i3, j3, k3);
    return _PerlinNoiseOperation.lerp(i4, j4, l3);
  }
};
PerlinNoiseOperation.initNoise();

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/PseudoRandomNoiseOperation.ts
var PseudoRandomNoiseOperation = class _PseudoRandomNoiseOperation extends TextureOperation {
  static noise(x, y) {
    let n = x + y * 57;
    n ^= n << 1;
    return 4096 - ((1376312589 + (789221 + 15731 * (n * n)) * n & 2147483647) / 262144 | 0);
  }
  constructor() {
    super(0, true);
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const vertGradient = textureGenerator.verticalGradient[line];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        const horzGradient = textureGenerator.horizontalGradient[pixel];
        output[pixel] = _PseudoRandomNoiseOperation.noise(horzGradient, vertGradient) % 4096;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/RangeOperation.ts
var RangeOperation = class extends TextureOperation {
  constructor() {
    super(1, false);
    this.field0 = 1024;
    this.field1 = 3072;
    this.field2 = this.field1 - this.field0;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.field0 = buffer.readUnsignedShort();
    } else if (field === 1) {
      this.field1 = buffer.readUnsignedShort();
    } else if (field === 2) {
      this.isMonochrome = buffer.readUnsignedByte() === 1;
    }
  }
  init() {
    this.field2 = this.field1 - this.field0;
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const input = this.getMonochromeInput(textureGenerator, 0, line);
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        output[pixel] = (this.field2 * input[pixel] >> 12) + this.field0;
      }
    }
    return output;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const input = this.getColourInput(textureGenerator, 0, line);
      const inputR = input[0];
      const inputG = input[1];
      const inputB = input[2];
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        outputR[pixel] = (this.field2 * inputR[pixel] >> 12) + this.field0;
        outputG[pixel] = (this.field2 * inputG[pixel] >> 12) + this.field0;
        outputB[pixel] = (this.field2 * inputB[pixel] >> 12) + this.field0;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/RasterizerOperation.ts
var RasterizerOperation = class extends TextureOperation {
  constructor() {
    super(0, true);
  }
  decode(field, buffer) {
    if (field === 0) {
      const count = buffer.readUnsignedByte();
      this.ops = new Array(count);
      for (let i = 0; i < count; i++) {
        const type = buffer.readUnsignedByte();
        if (type === 0) {
          this.ops[i] = RasterizerOperationLine.create(buffer);
        } else if (type === 1) {
          this.ops[i] = RasterizerOperationBezierCurve.create(buffer);
        } else if (type === 2) {
          this.ops[i] = RasterizerOperationRectangle.create(buffer);
        } else if (type === 3) {
          this.ops[i] = RasterizerOperationEllipse.create(buffer);
        }
      }
    } else if (field === 1) {
      this.isMonochrome = buffer.readUnsignedByte() === 1;
    }
  }
  render(textureGenerator, pixels) {
    const width = textureGenerator.width;
    const height = textureGenerator.height;
    Rasterizer.setPixels(pixels);
    Rasterizer.setDimensionMasks(textureGenerator.widthMask, textureGenerator.heightMask);
    if (this.ops === void 0) {
      return;
    }
    for (const op of this.ops) {
      const fillColor = op.fillColor;
      const outlineColor = op.outlineColor;
      if (fillColor >= 0) {
        if (outlineColor >= 0) {
          op.render(width, height);
        } else {
          op.renderFill(width, height);
        }
      } else if (outlineColor >= 0) {
        op.renderOutline(width, height);
      }
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      this.render(textureGenerator, this.monochromeImageCache.getAll());
    }
    return output;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const width = textureGenerator.width;
      const height = textureGenerator.height;
      const pixels = new Array(height);
      for (let i = 0; i < height; i++) {
        pixels[i] = new Int32Array(width);
      }
      const outputAll = this.colourImageCache.getAll();
      this.render(textureGenerator, pixels);
      for (let y = 0; y < textureGenerator.height; y++) {
        const output2 = outputAll[y];
        const outputR = output2[0];
        const outputG = output2[1];
        const outputB = output2[2];
        const input = pixels[y];
        for (let x = 0; x < textureGenerator.width; x++) {
          const rgb = input[x];
          outputR[x] = rgb >> 12 & 4080;
          outputG[x] = rgb >> 4 & 4080;
          outputB[x] = (rgb & 255) << 4;
        }
      }
    }
    return output;
  }
};
var Rasterizer = class _Rasterizer {
  static {
    this.widthMask = 0;
  }
  static {
    this.heightMask = 0;
  }
  static {
    this.startX = 0;
  }
  static {
    this.startY = 0;
  }
  static setPixels(pixels) {
    this.pixels = pixels;
  }
  static setDimensionMasks(widthMask, heightMask) {
    this.widthMask = widthMask;
    this.heightMask = heightMask;
    this.startX = 0;
    this.startY = 0;
  }
  static initCircleOutline(size) {
    if (this.circleOutline === void 0 || this.circleOutline.length < size) {
      this.circleOutline = new Int32Array(size);
    }
  }
  static rasterLine(x0, x1, y0, y1, color) {
    const deltaX = x1 - x0;
    const deltaY = y1 - y0;
    if (deltaX === 0) {
      if (deltaY !== 0) {
        _Rasterizer.rasterVerticalLine(x0, y0, y1, color);
      }
    } else if (deltaY === 0) {
      _Rasterizer.rasterHorizontalLine(x0, x1, y0, color);
    } else {
      const local55 = (deltaY << 12) / deltaX | 0;
      const local64 = y0 - (x0 * local55 >> 12);
      let startX;
      let startY;
      if (x0 < _Rasterizer.startX) {
        startY = local64 + (_Rasterizer.startX * local55 >> 12);
        startX = _Rasterizer.startX;
      } else if (_Rasterizer.widthMask >= x0) {
        startX = x0;
        startY = y0;
      } else {
        startX = _Rasterizer.widthMask;
        startY = (_Rasterizer.widthMask * local55 >> 12) + local64;
      }
      let endX;
      let endY;
      if (x1 < _Rasterizer.startX) {
        endX = _Rasterizer.startX;
        endY = local64 + (_Rasterizer.startX * local55 >> 12);
      } else if (x1 <= _Rasterizer.widthMask) {
        endX = x1;
        endY = y1;
      } else {
        endX = _Rasterizer.widthMask;
        endY = (local55 * _Rasterizer.widthMask >> 12) + local64;
      }
      if (_Rasterizer.startY > endY) {
        endX = (_Rasterizer.startY - local64 << 12) / local55;
        endY = _Rasterizer.startY;
      } else if (_Rasterizer.heightMask < endY) {
        endX = (_Rasterizer.heightMask - local64 << 12) / local55;
        endY = _Rasterizer.heightMask;
      }
      if (_Rasterizer.startY > startY) {
        startX = (_Rasterizer.startY - local64 << 12) / local55;
        startY = _Rasterizer.startY;
      } else if (startY > _Rasterizer.heightMask) {
        startY = _Rasterizer.heightMask;
        startX = (_Rasterizer.heightMask - local64 << 12) / local55;
      }
      _Rasterizer.rasterLine0(startX, endX, startY, endY, color);
    }
  }
  static rasterLine0(x0, x1, y0, y1, color) {
    let deltaX = x1 - x0;
    let deltaY = y1 - y0;
    if (deltaX === 0) {
      if (deltaY !== 0) {
        _Rasterizer.rasterVerticalLine0(x0, y0, y1, color);
      }
    } else if (deltaY === 0) {
      _Rasterizer.rasterHorizontalLine0(x0, x1, y0, color);
    } else {
      if (deltaX < 0) {
        deltaX = -deltaX;
      }
      if (deltaY < 0) {
        deltaY = -deltaY;
      }
      const local70 = deltaY > deltaX;
      if (local70) {
        const temp0 = x0;
        x0 = y0;
        y0 = temp0;
        const temp1 = x1;
        x1 = y1;
        y1 = temp1;
      }
      if (x1 < x0) {
        const temp0 = x0;
        x0 = x1;
        const temp1 = y0;
        y0 = y1;
        y1 = temp1;
        x1 = temp0;
      }
      let y = y0;
      const local110 = x1 - x0;
      let local115 = y1 - y0;
      const local126 = y1 > y0 ? 1 : -1;
      if (local115 < 0) {
        local115 = -local115;
      }
      let local137 = -(local110 >> 1);
      if (local70) {
        for (let local141 = x0; local141 <= x1; local141++) {
          local137 += local115;
          _Rasterizer.pixels[local141][y] = color;
          if (local137 > 0) {
            y += local126;
            local137 -= local110;
          }
        }
      } else {
        for (let x = x0; x <= x1; x++) {
          local137 += local115;
          _Rasterizer.pixels[y][x] = color;
          if (local137 > 0) {
            y += local126;
            local137 -= local110;
          }
        }
      }
    }
  }
  static rasterVerticalLine(x0, y0, y1, color) {
    if (_Rasterizer.startX <= x0 && _Rasterizer.widthMask >= x0) {
      y0 = clamp(y0, _Rasterizer.startY, _Rasterizer.heightMask);
      y1 = clamp(y1, _Rasterizer.startY, _Rasterizer.heightMask);
      _Rasterizer.rasterVerticalLine0(x0, y0, y1, color);
    }
  }
  static rasterVerticalLine0(x0, y0, y1, color) {
    if (y1 >= y0) {
      for (let y = y0; y < y1; y++) {
        _Rasterizer.pixels[y][x0] = color;
      }
    } else {
      for (let y = y1; y < y0; y++) {
        _Rasterizer.pixels[y][x0] = color;
      }
    }
  }
  static rasterHorizontalLine(x0, x1, y0, color) {
    if (_Rasterizer.startY <= y0 && y0 <= _Rasterizer.heightMask) {
      x0 = clamp(x0, _Rasterizer.startX, _Rasterizer.widthMask);
      x1 = clamp(x1, _Rasterizer.startX, _Rasterizer.widthMask);
      _Rasterizer.rasterHorizontalLine0(x0, x1, y0, color);
    }
  }
  static rasterHorizontalLine0(x0, x1, y0, color) {
    if (x1 >= x0) {
      ArrayUtils.fillRange(_Rasterizer.pixels[y0], x0, x1, color);
    } else {
      ArrayUtils.fillRange(_Rasterizer.pixels[y0], x1, x0, color);
    }
  }
  static rasterBezierCurve(x0, y0, x1, y1, x2, y2, x3, y3, outlineColor) {
    if (_Rasterizer.startX <= x0 && x0 <= _Rasterizer.widthMask && _Rasterizer.startX <= x1 && _Rasterizer.widthMask >= x1 && _Rasterizer.startX <= x2 && _Rasterizer.widthMask >= x2 && _Rasterizer.startX <= x3 && x3 <= _Rasterizer.widthMask && y0 >= _Rasterizer.startY && y0 <= _Rasterizer.heightMask && y1 >= _Rasterizer.startY && _Rasterizer.heightMask >= y1 && y2 >= _Rasterizer.startY && _Rasterizer.heightMask >= y2 && _Rasterizer.startY <= y3 && _Rasterizer.heightMask >= y3) {
      _Rasterizer.rasterBezierCurve0(x0, y0, x1, y1, x2, y2, x3, y3, outlineColor);
    } else {
      _Rasterizer.rasterBezierCurveClamped(x0, y0, x1, y1, x2, y2, x3, y3, outlineColor);
    }
  }
  static rasterBezierCurve0(x0, y0, x1, y1, x2, y2, x3, y3, outlineColor) {
    if (x0 === x1 && y0 === y1 && x2 === x3 && y2 === y3) {
      _Rasterizer.rasterLine0(x0, x3, y0, y3, outlineColor);
      return;
    }
    let local36 = x0;
    let local38 = y0;
    const local42 = x0 * 3;
    const local46 = y0 * 3;
    const local50 = x1 * 3;
    const local54 = x2 * 3;
    const local58 = y1 * 3;
    const local62 = y2 * 3;
    const local72 = local50 + x3 - x0 - local54;
    const local81 = local58 + y3 - local62 - y0;
    const local92 = local54 + local42 - local50 - local50;
    const local103 = local62 + local46 - local58 - local58;
    const local108 = local58 - local46;
    const local113 = local50 - local42;
    for (let local115 = 128; local115 <= 4096; local115 += 128) {
      const local126 = local115 * local115 >> 12;
      const local132 = local115 * local126 >> 12;
      const local136 = local132 * local72;
      const local140 = local126 * local92;
      const local144 = local113 * local115;
      const local148 = local103 * local126;
      const local158 = (local144 + local140 + local136 >> 12) + x0;
      const local162 = local81 * local132;
      const local166 = local108 * local115;
      const local176 = (local166 + local162 + local148 >> 12) + y0;
      _Rasterizer.rasterLine0(local36, local158, local38, local176, outlineColor);
      local36 = local158;
      local38 = local176;
    }
  }
  static rasterBezierCurveClamped(x0, y0, x1, y1, x2, y2, x3, y3, outlineColor) {
    if (x1 === x0 && y1 === y0 && x2 === x3 && y2 === y3) {
      _Rasterizer.rasterLine(x0, x3, y0, y3, outlineColor);
      return;
    }
    let local32 = x0;
    let local34 = y0;
    const local38 = x0 * 3;
    const local42 = y1 * 3;
    const local46 = x1 * 3;
    const local50 = x2 * 3;
    const local61 = x3 + local46 - x0 - local50;
    const local65 = y0 * 3;
    const local75 = local38 + local50 - local46 - local46;
    const local79 = y2 * 3;
    const local90 = local79 + local65 - local42 - local42;
    const local100 = local42 + y3 - y0 - local79;
    const local104 = local46 - local38;
    const local108 = local42 - local65;
    for (let local110 = 128; local110 <= 4096; local110 += 128) {
      const local119 = local110 * local110 >> 12;
      const local123 = local119 * local75;
      const local129 = local110 * local119 >> 12;
      const local133 = local129 * local61;
      const local137 = local108 * local110;
      const local141 = local90 * local119;
      const local145 = local100 * local129;
      const local149 = local104 * local110;
      const local159 = (local149 + local123 + local133 >> 12) + x0;
      const local170 = (local137 + local145 + local141 >> 12) + y0;
      _Rasterizer.rasterLine(local32, local159, local34, local170, outlineColor);
      local34 = local170;
      local32 = local159;
    }
  }
  static rasterRectangle(x0, x1, y0, y1, fillColor, outlineColor, outlineWidth) {
    if (x0 >= _Rasterizer.startX && x1 <= _Rasterizer.widthMask && _Rasterizer.startY <= y0 && y1 <= _Rasterizer.heightMask) {
      _Rasterizer.rasterRectangle0(x0, x1, y0, y1, fillColor, outlineColor, outlineWidth);
    } else {
      _Rasterizer.rasterRectangleClamped(
        x0,
        x1,
        y0,
        y1,
        fillColor,
        outlineColor,
        outlineWidth
      );
    }
  }
  static rasterRectangle0(x0, x1, y0, y1, fillColor, outlineColor, outlineWidth) {
    const local6 = y0 + outlineWidth;
    const local14 = y1 - outlineWidth;
    const local18 = x0 + outlineWidth;
    const local23 = x1 - outlineWidth;
    for (let local25 = y0; local25 < local6; local25++) {
      ArrayUtils.fillRange(_Rasterizer.pixels[local25], x0, x1, outlineColor);
    }
    for (let local55 = y1; local55 > local14; local55--) {
      ArrayUtils.fillRange(_Rasterizer.pixels[local55], x0, x1, outlineColor);
    }
    for (let local75 = local6; local75 <= local14; local75++) {
      const local86 = _Rasterizer.pixels[local75];
      ArrayUtils.fillRange(local86, x0, local18, outlineColor);
      ArrayUtils.fillRange(local86, local18, local23, fillColor);
      ArrayUtils.fillRange(local86, local23, x1, outlineColor);
    }
  }
  static rasterRectangleClamped(x0, x1, y0, y1, fillColor, outlineColor, outlineWidth) {
    const local11 = clamp(y0, _Rasterizer.startY, _Rasterizer.heightMask);
    const local22 = clamp(y1, _Rasterizer.startY, _Rasterizer.heightMask);
    const local28 = clamp(x0, _Rasterizer.startX, _Rasterizer.widthMask);
    const local34 = clamp(x1, _Rasterizer.startX, _Rasterizer.widthMask);
    const local43 = clamp(y0 + outlineWidth, _Rasterizer.startY, _Rasterizer.heightMask);
    const local52 = clamp(y1 - outlineWidth, _Rasterizer.startY, _Rasterizer.heightMask);
    for (let local54 = local11; local54 < local43; local54++) {
      ArrayUtils.fillRange(_Rasterizer.pixels[local54], local28, local34, outlineColor);
    }
    for (let local74 = local22; local74 > local52; local74--) {
      ArrayUtils.fillRange(_Rasterizer.pixels[local74], local28, local34, outlineColor);
    }
    const local97 = clamp(x0 + outlineWidth, _Rasterizer.startX, _Rasterizer.widthMask);
    const local106 = clamp(x1 - outlineWidth, _Rasterizer.startX, _Rasterizer.widthMask);
    for (let local108 = local43; local108 <= local52; local108++) {
      const local119 = _Rasterizer.pixels[local108];
      ArrayUtils.fillRange(local119, local28, local97, outlineColor);
      ArrayUtils.fillRange(local119, local97, local106, fillColor);
      ArrayUtils.fillRange(local119, local106, local34, outlineColor);
    }
  }
  static rasterRectangleFill(x0, x1, y0, y1, fillColor) {
    if (_Rasterizer.startX <= x0 && x1 <= _Rasterizer.widthMask && _Rasterizer.startY <= y0 && y1 <= _Rasterizer.heightMask) {
      _Rasterizer.rasterRectangleFill0(x0, x1, y0, y1, fillColor);
    } else {
      _Rasterizer.rasterRectangleFillClamped(x0, x1, y0, y1, fillColor);
    }
  }
  static rasterRectangleFill0(x0, x1, y0, y1, fillColor) {
    for (let local6 = y0; local6 <= y1; local6++) {
      ArrayUtils.fillRange(_Rasterizer.pixels[local6], x0, x1, fillColor);
    }
  }
  static rasterRectangleFillClamped(x0, x1, y0, y1, fillColor) {
    const local11 = clamp(y0, _Rasterizer.startY, _Rasterizer.heightMask);
    const local17 = clamp(y1, _Rasterizer.startY, _Rasterizer.heightMask);
    const local23 = clamp(x0, _Rasterizer.startX, _Rasterizer.widthMask);
    const local29 = clamp(x1, _Rasterizer.startX, _Rasterizer.widthMask);
    for (let local31 = local11; local31 <= local17; local31++) {
      ArrayUtils.fillRange(_Rasterizer.pixels[local31], local23, local29, fillColor);
    }
  }
  static rasterRectangleOutline(x0, x1, y0, y1, outlineColor, outlineWidth) {
    if (x0 >= _Rasterizer.startX && _Rasterizer.widthMask >= x1 && y0 >= _Rasterizer.startY && _Rasterizer.heightMask >= y1) {
      if (outlineWidth === 1) {
        _Rasterizer.rasterRectangleOutlineWidth1(x0, x1, y0, y1, outlineColor);
      } else {
        _Rasterizer.rasterRectangleOutline0(x0, x1, y0, y1, outlineColor, outlineWidth);
      }
    } else if (outlineWidth === 1) {
      _Rasterizer.rasterRectangleOutlineWidth1Clamped(x0, x1, y0, y1, outlineColor);
    } else {
      _Rasterizer.rasterRectangleOutlineClamped(x0, x1, y0, y1, outlineColor, outlineWidth);
    }
  }
  static rasterRectangleOutlineWidth1(x0, x1, y0, y1, outlineColor) {
    ArrayUtils.fillRange(_Rasterizer.pixels[y0++], x0, x1, outlineColor);
    ArrayUtils.fillRange(_Rasterizer.pixels[y1--], x0, x1, outlineColor);
    for (let local31 = y0; local31 <= y1; local31++) {
      const local42 = _Rasterizer.pixels[local31];
      local42[x0] = local42[x1] = outlineColor;
    }
  }
  static rasterRectangleOutline0(x0, x1, y0, y1, outlineColor, outlineWidth) {
    const local10 = outlineWidth + y0;
    const local18 = y1 - outlineWidth;
    const local22 = outlineWidth + x0;
    for (let local24 = y0; local24 < local10; local24++) {
      ArrayUtils.fillRange(_Rasterizer.pixels[local24], x0, x1, outlineColor);
    }
    for (let local44 = y1; local44 > local18; local44--) {
      ArrayUtils.fillRange(_Rasterizer.pixels[local44], x0, x1, outlineColor);
    }
    const local66 = x1 - outlineWidth;
    for (let local68 = local10; local68 <= local18; local68++) {
      const local79 = _Rasterizer.pixels[local68];
      ArrayUtils.fillRange(local79, x0, local22, outlineColor);
      ArrayUtils.fillRange(local79, local66, x1, outlineColor);
    }
  }
  static rasterRectangleOutlineWidth1Clamped(x0, x1, y0, y1, outlineColor) {
    if (_Rasterizer.heightMask < y0 || _Rasterizer.startY > y1) {
      return;
    }
    let local23;
    if (_Rasterizer.startX > x0) {
      x0 = _Rasterizer.startX;
      local23 = false;
    } else if (x0 > _Rasterizer.widthMask) {
      x0 = _Rasterizer.widthMask;
      local23 = false;
    } else {
      local23 = true;
    }
    let local51;
    if (x1 < _Rasterizer.startX) {
      x1 = _Rasterizer.startX;
      local51 = false;
    } else if (_Rasterizer.widthMask < x1) {
      x1 = _Rasterizer.widthMask;
      local51 = false;
    } else {
      local51 = true;
    }
    let local71;
    if (_Rasterizer.startY <= y0) {
      local71 = y0 + 1;
      ArrayUtils.fillRange(_Rasterizer.pixels[y0], x0, x1, outlineColor);
    } else {
      local71 = _Rasterizer.startY;
    }
    let local89;
    if (_Rasterizer.heightMask < y1) {
      local89 = _Rasterizer.heightMask;
    } else {
      local89 = y1 - 1;
      ArrayUtils.fillRange(_Rasterizer.pixels[y1], x0, x1, outlineColor);
    }
    if (local23 && local51) {
      for (let local106 = local71; local106 <= local89; local106++) {
        const local113 = _Rasterizer.pixels[local106];
        local113[x0] = local113[x1] = outlineColor;
      }
    } else if (local23) {
      for (let local149 = local71; local149 <= local89; local149++) {
        _Rasterizer.pixels[local149][x0] = outlineColor;
      }
    } else if (local51) {
      for (let local133 = local71; local133 <= local89; local133++) {
        _Rasterizer.pixels[local133][x1] = outlineColor;
      }
    }
  }
  static rasterRectangleOutlineClamped(x0, x1, y0, y1, outlineColor, outlineWidth) {
    const local17 = clamp(y0, _Rasterizer.startY, _Rasterizer.heightMask);
    const local23 = clamp(y1, _Rasterizer.startY, _Rasterizer.heightMask);
    const local29 = clamp(x0, _Rasterizer.startX, _Rasterizer.widthMask);
    const local35 = clamp(x1, _Rasterizer.startX, _Rasterizer.widthMask);
    const local43 = clamp(outlineWidth + y0, _Rasterizer.startY, _Rasterizer.heightMask);
    const local52 = clamp(y1 - outlineWidth, _Rasterizer.startY, _Rasterizer.heightMask);
    for (let local54 = local17; local54 < local43; local54++) {
      ArrayUtils.fillRange(_Rasterizer.pixels[local54], local29, local35, outlineColor);
    }
    for (let local70 = local23; local70 > local52; local70--) {
      ArrayUtils.fillRange(_Rasterizer.pixels[local70], local29, local35, outlineColor);
    }
    const local97 = clamp(x0 + outlineWidth, _Rasterizer.startX, _Rasterizer.widthMask);
    const local106 = clamp(x1 - outlineWidth, _Rasterizer.startX, _Rasterizer.widthMask);
    for (let local108 = local43; local108 <= local52; local108++) {
      const local119 = _Rasterizer.pixels[local108];
      ArrayUtils.fillRange(local119, local29, local97, outlineColor);
      ArrayUtils.fillRange(local119, local106, local35, outlineColor);
    }
  }
  static rasterEllipse(x, y, sizeX, sizeY, fillColor, outlineColor, outlineWidth) {
    if (sizeX === sizeY) {
      _Rasterizer.rasterCircle(x, y, sizeX, fillColor, outlineColor, outlineWidth);
    } else if (x - sizeX >= _Rasterizer.startX && _Rasterizer.widthMask >= sizeX + x && y - sizeY >= _Rasterizer.startY && sizeY + y <= _Rasterizer.heightMask) {
      _Rasterizer.rasterEllipse0(x, y, sizeX, sizeY, fillColor, outlineColor, outlineWidth);
    } else {
      _Rasterizer.rasterEllipseClamped(
        x,
        y,
        sizeX,
        sizeY,
        fillColor,
        outlineColor,
        outlineWidth
      );
    }
  }
  static rasterEllipse0(x, y, sizeX, sizeY, fillColor, outlineColor, outlineWidth) {
    let local7 = 0;
    let local9 = sizeY;
    const local14 = sizeX - outlineWidth;
    let local16 = 0;
    const local21 = sizeY - outlineWidth;
    const local25 = sizeX * sizeX;
    const local29 = sizeY * sizeY;
    const local33 = local14 * local14;
    const local37 = local21 * local21;
    const local41 = local29 << 1;
    const local45 = local37 << 1;
    const local49 = local25 << 1;
    const local53 = local33 << 1;
    const local57 = sizeY << 1;
    const local61 = local21 << 1;
    let local71 = local25 * (1 - local57) + local41;
    let local80 = local29 - local49 * (local57 - 1);
    let local89 = local45 + (1 - local61) * local33;
    let local98 = local37 - local53 * (local61 - 1);
    const local102 = local25 << 2;
    const local106 = local29 << 2;
    const local110 = local37 << 2;
    const local114 = local33 << 2;
    let local118 = local41 * 3;
    let local124 = local49 * (local57 - 3);
    let local128 = local45 * 3;
    let local130 = local106;
    let local136 = (local61 - 3) * local53;
    let local138 = local110;
    let local144 = (sizeY - 1) * local102;
    let local150 = local114 * (local21 - 1);
    const local154 = _Rasterizer.pixels[y];
    ArrayUtils.fillRange(local154, x - sizeX, x - local14, outlineColor);
    ArrayUtils.fillRange(local154, x - local14, local14 + x, fillColor);
    ArrayUtils.fillRange(local154, local14 + x, x + sizeX, outlineColor);
    while (local9 > 0) {
      if (local71 < 0) {
        while (local71 < 0) {
          local71 += local118;
          local118 += local106;
          local7++;
          local80 += local130;
          local130 += local106;
        }
      }
      if (local80 < 0) {
        local71 += local118;
        local80 += local130;
        local118 += local106;
        local7++;
        local130 += local106;
      }
      local71 += -local144;
      const local251 = x - local7;
      const local258 = local21 >= local9;
      const local263 = x + local7;
      local144 -= local102;
      local9--;
      local80 += -local124;
      const local277 = local9 + y;
      local124 -= local102;
      if (local258) {
        if (local89 < 0) {
          while (local89 < 0) {
            local16++;
            local98 += local138;
            local89 += local128;
            local138 += local110;
            local128 += local110;
          }
        }
        if (local98 < 0) {
          local89 += local128;
          local128 += local110;
          local16++;
          local98 += local138;
          local138 += local110;
        }
        local98 += -local136;
        local89 += -local150;
        local150 -= local114;
        local136 -= local114;
      }
      const local352 = y - local9;
      if (local258) {
        const local358 = x - local16;
        ArrayUtils.fillRange(_Rasterizer.pixels[local352], local251, local358, outlineColor);
        const local371 = x + local16;
        ArrayUtils.fillRange(_Rasterizer.pixels[local352], local358, local371, fillColor);
        ArrayUtils.fillRange(_Rasterizer.pixels[local352], local371, local263, outlineColor);
        ArrayUtils.fillRange(_Rasterizer.pixels[local277], local251, local358, outlineColor);
        ArrayUtils.fillRange(_Rasterizer.pixels[local277], local358, local371, fillColor);
        ArrayUtils.fillRange(_Rasterizer.pixels[local277], local371, local263, outlineColor);
      } else {
        ArrayUtils.fillRange(_Rasterizer.pixels[local352], local251, local263, outlineColor);
        ArrayUtils.fillRange(_Rasterizer.pixels[local277], local251, local263, outlineColor);
      }
    }
  }
  static rasterEllipseClamped(x, y, sizeX, sizeY, fillColor, outlineColor, outlineWidth) {
    let local3 = 0;
    const local7 = sizeY - outlineWidth;
    let local9 = sizeY;
    let local11 = 0;
    const local16 = sizeX - outlineWidth;
    const local20 = sizeX * sizeX;
    const local28 = sizeY * sizeY;
    const local32 = local16 * local16;
    const local36 = local28 << 1;
    const local40 = local7 * local7;
    const local44 = local20 << 1;
    const local48 = local40 << 1;
    const local52 = local32 << 1;
    const local56 = sizeY << 1;
    const local60 = local7 << 1;
    let local69 = local36 + (1 - local56) * local20;
    let local78 = local28 - (local56 - 1) * local44;
    let local87 = local32 * (1 - local60) + local48;
    const local91 = local20 << 2;
    let local104 = local40 - local52 * (local60 - 1);
    const local108 = local28 << 2;
    const local112 = local32 << 2;
    const local116 = local40 << 2;
    let local120 = local36 * 3;
    let local124 = local48 * 3;
    let local130 = local44 * (local56 - 3);
    let local132 = local108;
    let local138 = (local60 - 3) * local52;
    let local144 = local91 * (sizeY - 1);
    let local146 = local116;
    let local152 = (local7 - 1) * local112;
    if (y >= _Rasterizer.startY && _Rasterizer.heightMask >= y) {
      const local166 = _Rasterizer.pixels[y];
      const local177 = clamp(x - sizeX, _Rasterizer.startX, _Rasterizer.widthMask);
      const local185 = clamp(x + sizeX, _Rasterizer.startX, _Rasterizer.widthMask);
      const local193 = clamp(x - local16, _Rasterizer.startX, _Rasterizer.widthMask);
      const local203 = clamp(x + local16, _Rasterizer.startX, _Rasterizer.widthMask);
      ArrayUtils.fillRange(local166, local177, local193, outlineColor);
      ArrayUtils.fillRange(local166, local193, local203, fillColor);
      ArrayUtils.fillRange(local166, local203, local185, outlineColor);
    }
    while (local9 > 0) {
      if (local69 < 0) {
        while (local69 < 0) {
          local69 += local120;
          local3++;
          local120 += local108;
          local78 += local132;
          local132 += local108;
        }
      }
      if (local78 < 0) {
        local3++;
        local69 += local120;
        local120 += local108;
        local78 += local132;
        local132 += local108;
      }
      const local281 = local7 >= local9;
      local78 += -local130;
      local9--;
      if (local281) {
        if (local87 < 0) {
          while (local87 < 0) {
            local87 += local124;
            local104 += local146;
            local124 += local116;
            local146 += local116;
            local11++;
          }
        }
        if (local104 < 0) {
          local87 += local124;
          local124 += local116;
          local104 += local146;
          local146 += local116;
          local11++;
        }
        local104 += -local138;
        local138 -= local112;
        local87 += -local152;
        local152 -= local112;
      }
      local69 += -local144;
      const local363 = local9 + y;
      const local367 = y - local9;
      local144 -= local91;
      local130 -= local91;
      if (local363 >= _Rasterizer.startY && _Rasterizer.heightMask >= local367) {
        const local389 = clamp(local3 + x, _Rasterizer.startX, _Rasterizer.widthMask);
        const local398 = clamp(x - local3, _Rasterizer.startX, _Rasterizer.widthMask);
        if (local281) {
          const local409 = clamp(x + local11, _Rasterizer.startX, _Rasterizer.widthMask);
          const local418 = clamp(x - local11, _Rasterizer.startX, _Rasterizer.widthMask);
          if (_Rasterizer.startY <= local367) {
            const local426 = _Rasterizer.pixels[local367];
            ArrayUtils.fillRange(local426, local398, local418, outlineColor);
            ArrayUtils.fillRange(local426, local418, local409, fillColor);
            ArrayUtils.fillRange(local426, local409, local389, outlineColor);
          }
          if (_Rasterizer.heightMask >= local363) {
            const local456 = _Rasterizer.pixels[local363];
            ArrayUtils.fillRange(local456, local398, local418, outlineColor);
            ArrayUtils.fillRange(local456, local418, local409, fillColor);
            ArrayUtils.fillRange(local456, local409, local389, outlineColor);
          }
        } else {
          if (local367 >= _Rasterizer.startY) {
            ArrayUtils.fillRange(
              _Rasterizer.pixels[local367],
              local398,
              local389,
              outlineColor
            );
          }
          if (_Rasterizer.heightMask >= local363) {
            ArrayUtils.fillRange(
              _Rasterizer.pixels[local363],
              local398,
              local389,
              outlineColor
            );
          }
        }
      }
    }
  }
  static rasterCircle(x, y, size, fillColor, outlineColor, outlineWidth) {
    if (_Rasterizer.startX <= x - size && size + x <= _Rasterizer.widthMask && y - size >= _Rasterizer.startY && _Rasterizer.heightMask >= size + y) {
      _Rasterizer.rasterCircle0(x, y, size, fillColor, outlineColor, outlineWidth);
    } else {
      _Rasterizer.rasterCircleClamped(x, y, size, fillColor, outlineColor, outlineWidth);
    }
  }
  static rasterCircle0(x, y, size, fillColor, outlineColor, outlineWidth) {
    _Rasterizer.initCircleOutline(size);
    let local10 = 0;
    let local13 = -size;
    let local17 = size - outlineWidth;
    let local19 = size;
    let local21 = -1;
    let local23 = -1;
    const local27 = _Rasterizer.pixels[y];
    if (local17 < 0) {
      local17 = 0;
    }
    let local34 = local17;
    const local39 = x - local17;
    ArrayUtils.fillRange(local27, x - size, local39, outlineColor);
    const local57 = local17 + x;
    let local60 = -local17;
    ArrayUtils.fillRange(local27, local39, local57, fillColor);
    ArrayUtils.fillRange(local27, local57, x + size, outlineColor);
    while (local19 > local10) {
      local23 += 2;
      local13 += local23;
      local21 += 2;
      local60 += local21;
      if (local60 >= 0 && local34 >= 1) {
        _Rasterizer.circleOutline[local34] = local10;
        local34--;
        local60 -= local34 << 1;
      }
      local10++;
      if (local13 >= 0) {
        local19--;
        if (local19 >= local17) {
          const local126 = _Rasterizer.pixels[y + local19];
          const local131 = x + local10;
          const local138 = _Rasterizer.pixels[y - local19];
          const local143 = x - local10;
          ArrayUtils.fillRange(local126, local143, local131, outlineColor);
          ArrayUtils.fillRange(local138, local143, local131, outlineColor);
        } else {
          const local163 = _Rasterizer.pixels[local19 + y];
          const local167 = _Rasterizer.circleOutline[local19];
          const local174 = _Rasterizer.pixels[y - local19];
          const local178 = local10 + x;
          const local182 = x - local167;
          const local186 = local167 + x;
          const local191 = x - local10;
          ArrayUtils.fillRange(local163, local191, local182, outlineColor);
          ArrayUtils.fillRange(local163, local182, local186, fillColor);
          ArrayUtils.fillRange(local163, local186, local178, outlineColor);
          ArrayUtils.fillRange(local174, local191, local182, outlineColor);
          ArrayUtils.fillRange(local174, local182, local186, fillColor);
          ArrayUtils.fillRange(local174, local186, local178, outlineColor);
        }
        local13 -= local19 << 1;
      }
      const local240 = _Rasterizer.pixels[y + local10];
      const local247 = _Rasterizer.pixels[y - local10];
      const local251 = local19 + x;
      const local256 = x - local19;
      if (local17 <= local10) {
        ArrayUtils.fillRange(local240, local256, local251, outlineColor);
        ArrayUtils.fillRange(local247, local256, local251, outlineColor);
      } else {
        const local286 = local10 > local34 ? _Rasterizer.circleOutline[local10] : local34;
        const local290 = local286 + x;
        const local294 = x - local286;
        ArrayUtils.fillRange(local240, local256, local294, outlineColor);
        ArrayUtils.fillRange(local240, local294, local290, fillColor);
        ArrayUtils.fillRange(local240, local290, local251, outlineColor);
        ArrayUtils.fillRange(local247, local256, local294, outlineColor);
        ArrayUtils.fillRange(local247, local294, local290, fillColor);
        ArrayUtils.fillRange(local247, local290, local251, outlineColor);
      }
    }
  }
  static rasterCircleClamped(x, y, size, fillColor, outlineColor, outlineWidth) {
    _Rasterizer.initCircleOutline(size);
    let local9 = size - outlineWidth;
    let local15 = size;
    let local17 = 0;
    let local20 = -size;
    if (local9 < 0) {
      local9 = 0;
    }
    let local29 = local9;
    if (_Rasterizer.startY <= y && y <= _Rasterizer.heightMask) {
      const local40 = _Rasterizer.pixels[y];
      const local48 = clamp(x - size, _Rasterizer.startX, _Rasterizer.widthMask);
      const local56 = clamp(size + x, _Rasterizer.startX, _Rasterizer.widthMask);
      const local67 = clamp(x - local9, _Rasterizer.startX, _Rasterizer.widthMask);
      const local77 = clamp(x + local9, _Rasterizer.startX, _Rasterizer.widthMask);
      ArrayUtils.fillRange(local40, local48, local67, outlineColor);
      ArrayUtils.fillRange(local40, local67, local77, fillColor);
      ArrayUtils.fillRange(local40, local77, local56, outlineColor);
    }
    let local98 = -local9;
    let local100 = -1;
    let local102 = -1;
    while (local17 < local15) {
      local100 += 2;
      local98 += local100;
      local102 += 2;
      if (local98 >= 0 && local29 >= 1) {
        local29--;
        local98 -= local29 << 1;
        _Rasterizer.circleOutline[local29] = local17;
      }
      local17++;
      local20 += local102;
      if (local20 >= 0) {
        local15--;
        local20 -= local15 << 1;
        const local154 = y - local15;
        const local159 = y + local15;
        if (_Rasterizer.startY <= local159 && local154 <= _Rasterizer.heightMask) {
          if (local15 >= local9) {
            const local186 = clamp(
              x + local17,
              _Rasterizer.startX,
              _Rasterizer.widthMask
            );
            const local194 = clamp(
              x - local17,
              _Rasterizer.startX,
              _Rasterizer.widthMask
            );
            if (_Rasterizer.heightMask >= local159) {
              ArrayUtils.fillRange(
                _Rasterizer.pixels[local159],
                local194,
                local186,
                outlineColor
              );
            }
            if (local154 >= _Rasterizer.startY) {
              ArrayUtils.fillRange(
                _Rasterizer.pixels[local154],
                local194,
                local186,
                outlineColor
              );
            }
          } else {
            const local226 = _Rasterizer.circleOutline[local15];
            const local237 = clamp(
              x + local17,
              _Rasterizer.startX,
              _Rasterizer.widthMask
            );
            const local245 = clamp(
              x - local17,
              _Rasterizer.startX,
              _Rasterizer.widthMask
            );
            const local254 = clamp(
              x + local226,
              _Rasterizer.startX,
              _Rasterizer.widthMask
            );
            const local262 = clamp(
              x - local226,
              _Rasterizer.startX,
              _Rasterizer.widthMask
            );
            if (_Rasterizer.heightMask >= local159) {
              const local274 = _Rasterizer.pixels[local159];
              ArrayUtils.fillRange(local274, local245, local262, outlineColor);
              ArrayUtils.fillRange(local274, local262, local254, fillColor);
              ArrayUtils.fillRange(local274, local254, local237, outlineColor);
            }
            if (_Rasterizer.startY <= local154) {
              const local300 = _Rasterizer.pixels[local154];
              ArrayUtils.fillRange(local300, local245, local262, outlineColor);
              ArrayUtils.fillRange(local300, local262, local254, fillColor);
              ArrayUtils.fillRange(local300, local254, local237, outlineColor);
            }
          }
        }
      }
      const local322 = y + local17;
      const local327 = y - local17;
      if (_Rasterizer.startY <= local322 && _Rasterizer.heightMask >= local327) {
        const local337 = local15 + x;
        const local342 = x - local15;
        if (local337 >= _Rasterizer.startX && _Rasterizer.widthMask >= local342) {
          const local359 = clamp(local337, _Rasterizer.startX, _Rasterizer.widthMask);
          const local365 = clamp(local342, _Rasterizer.startX, _Rasterizer.widthMask);
          if (local17 >= local9) {
            if (_Rasterizer.heightMask >= local322) {
              ArrayUtils.fillRange(
                _Rasterizer.pixels[local322],
                local365,
                local359,
                outlineColor
              );
            }
            if (_Rasterizer.startY <= local327) {
              ArrayUtils.fillRange(
                _Rasterizer.pixels[local327],
                local365,
                local359,
                outlineColor
              );
            }
          } else {
            const local415 = local29 >= local17 ? local29 : _Rasterizer.circleOutline[local17];
            const local424 = clamp(
              x + local415,
              _Rasterizer.startX,
              _Rasterizer.widthMask
            );
            const local432 = clamp(
              x - local415,
              _Rasterizer.startX,
              _Rasterizer.widthMask
            );
            if (_Rasterizer.heightMask >= local322) {
              const local440 = _Rasterizer.pixels[local322];
              ArrayUtils.fillRange(local440, local365, local432, outlineColor);
              ArrayUtils.fillRange(local440, local432, local424, fillColor);
              ArrayUtils.fillRange(local440, local424, local359, outlineColor);
            }
            if (_Rasterizer.startY <= local327) {
              const local469 = _Rasterizer.pixels[local327];
              ArrayUtils.fillRange(local469, local365, local432, outlineColor);
              ArrayUtils.fillRange(local469, local432, local424, fillColor);
              ArrayUtils.fillRange(local469, local424, local359, outlineColor);
            }
          }
        }
      }
    }
  }
  static rasterEllipseFill(x, y, sizeX, sizeY, fillColor) {
    if (sizeX === sizeY) {
      _Rasterizer.rasterCircleFill(x, y, sizeX, fillColor);
    } else if (_Rasterizer.startX <= x - sizeX && _Rasterizer.widthMask >= x + sizeX && y - sizeY >= _Rasterizer.startY && _Rasterizer.heightMask >= y + sizeY) {
      _Rasterizer.rasterEllipseFill0(x, y, sizeX, sizeY, fillColor);
    } else {
      _Rasterizer.rasterEllipseFillClamped(x, y, sizeX, sizeY, fillColor);
    }
  }
  static rasterEllipseFill0(x, y, sizeX, sizeY, fillColor) {
    ArrayUtils.fillRange(_Rasterizer.pixels[y], x - sizeX, sizeX + x, fillColor);
    let local20 = 0;
    let local22 = sizeY;
    const local26 = sizeX * sizeX;
    const local34 = sizeY * sizeY;
    const local38 = local26 << 1;
    const local42 = sizeY << 1;
    const local46 = local34 << 1;
    let local54 = local34 - local38 * (local42 - 1);
    let local63 = local46 + (1 - local42) * local26;
    const local67 = local26 << 2;
    let local75 = local46 * 3;
    let local83 = local38 * ((sizeY << 1) - 3);
    const local87 = local34 << 2;
    let local93 = local87;
    let local99 = (sizeY - 1) * local67;
    while (local22 > 0) {
      if (local63 < 0) {
        while (local63 < 0) {
          local20++;
          local63 += local75;
          local54 += local93;
          local93 += local87;
          local75 += local87;
        }
      }
      local22--;
      if (local54 < 0) {
        local54 += local93;
        local63 += local75;
        local75 += local87;
        local93 += local87;
        local20++;
      }
      const local150 = y - local22;
      local63 += -local99;
      const local159 = x + local20;
      local99 -= local67;
      const local167 = local22 + y;
      local54 += -local83;
      local83 -= local67;
      const local181 = x - local20;
      ArrayUtils.fillRange(_Rasterizer.pixels[local150], local181, local159, fillColor);
      ArrayUtils.fillRange(_Rasterizer.pixels[local167], local181, local159, fillColor);
    }
  }
  static rasterEllipseFillClamped(x, y, sizeX, sizeY, fillColor) {
    let local7 = sizeY;
    let local9 = 0;
    const local21 = sizeY * sizeY;
    const local25 = sizeX * sizeX;
    const local29 = local25 << 1;
    const local33 = local21 << 1;
    const local37 = sizeY << 1;
    let local46 = local21 - (local37 - 1) * local29;
    let local56 = (1 - local37) * local25 + local33;
    const local60 = local25 << 2;
    const local64 = local21 << 2;
    let local72 = local33 * 3;
    let local78 = local64;
    let local86 = ((sizeY << 1) - 3) * local29;
    if (y >= _Rasterizer.startY && _Rasterizer.heightMask >= y) {
      const local109 = clamp(x + sizeX, _Rasterizer.startX, _Rasterizer.widthMask);
      const local117 = clamp(x - sizeX, _Rasterizer.startX, _Rasterizer.widthMask);
      ArrayUtils.fillRange(_Rasterizer.pixels[y], local117, local109, fillColor);
    }
    let local131 = local60 * (sizeY - 1);
    while (local7 > 0) {
      if (local56 < 0) {
        while (local56 < 0) {
          local56 += local72;
          local46 += local78;
          local78 += local64;
          local72 += local64;
          local9++;
        }
      }
      local7--;
      if (local46 < 0) {
        local46 += local78;
        local78 += local64;
        local56 += local72;
        local9++;
        local72 += local64;
      }
      local56 += -local131;
      const local198 = y - local7;
      local46 += -local86;
      local131 -= local60;
      const local211 = local7 + y;
      local86 -= local60;
      if (local211 >= _Rasterizer.startY && _Rasterizer.heightMask >= local198) {
        const local229 = clamp(local9 + x, _Rasterizer.startX, _Rasterizer.widthMask);
        const local237 = clamp(x - local9, _Rasterizer.startX, _Rasterizer.widthMask);
        if (_Rasterizer.startY <= local198) {
          ArrayUtils.fillRange(
            _Rasterizer.pixels[local198],
            local237,
            local229,
            fillColor
          );
        }
        if (local211 <= _Rasterizer.heightMask) {
          ArrayUtils.fillRange(
            _Rasterizer.pixels[local211],
            local237,
            local229,
            fillColor
          );
        }
      }
    }
  }
  static rasterCircleFill(x, y, size, fillColor) {
    if (x - size >= _Rasterizer.startX && _Rasterizer.widthMask >= x + size && y - size >= _Rasterizer.startY && y + size <= _Rasterizer.heightMask) {
      _Rasterizer.rasterCircleFill0(x, y, size, fillColor);
    } else {
      _Rasterizer.rasterCircleFillClamped(x, y, size, fillColor);
    }
  }
  static rasterCircleFill0(x, y, size, fillColor) {
    ArrayUtils.fillRange(_Rasterizer.pixels[y], x - size, size + x, fillColor);
    let local20 = 0;
    let local33 = size;
    let local36 = -size;
    let local38 = -1;
    while (local33 > local20) {
      local38 += 2;
      local20++;
      local36 += local38;
      if (local36 >= 0) {
        local33--;
        local36 -= local33 << 1;
        const local69 = _Rasterizer.pixels[y - local33];
        const local76 = _Rasterizer.pixels[y + local33];
        const local80 = x - local20;
        const local84 = x + local20;
        ArrayUtils.fillRange(local76, local80, local84, fillColor);
        ArrayUtils.fillRange(local69, local80, local84, fillColor);
      }
      const local101 = x + local33;
      const local106 = x - local33;
      const local112 = _Rasterizer.pixels[y + local20];
      const local118 = _Rasterizer.pixels[y - local20];
      ArrayUtils.fillRange(local112, local106, local101, fillColor);
      ArrayUtils.fillRange(local118, local106, local101, fillColor);
    }
  }
  static rasterCircleFillClamped(x, y, size, fillColor) {
    let local3 = 0;
    let local14 = size;
    let local16 = -1;
    let local19 = -size;
    let local27 = clamp(size + x, _Rasterizer.startX, _Rasterizer.widthMask);
    let local35 = clamp(x - size, _Rasterizer.startX, _Rasterizer.widthMask);
    ArrayUtils.fillRange(_Rasterizer.pixels[y], local35, local27, fillColor);
    while (local14 > local3) {
      local16 += 2;
      local19 += local16;
      if (local19 > 0) {
        local14--;
        local19 -= local14 << 1;
        const local72 = y - local14;
        const local76 = local14 + y;
        if (local76 >= _Rasterizer.startY && local72 <= _Rasterizer.heightMask) {
          let local98 = clamp(x + local3, _Rasterizer.startX, _Rasterizer.widthMask);
          let local106 = clamp(x - local3, _Rasterizer.startX, _Rasterizer.widthMask);
          if (_Rasterizer.heightMask >= local76) {
            ArrayUtils.fillRange(
              _Rasterizer.pixels[local76],
              local106,
              local98,
              fillColor
            );
          }
          if (_Rasterizer.startY <= local72) {
            ArrayUtils.fillRange(
              _Rasterizer.pixels[local72],
              local106,
              local98,
              fillColor
            );
          }
        }
      }
      local3++;
      const local138 = y - local3;
      const local142 = y + local3;
      if (_Rasterizer.startY <= local142 && local138 <= _Rasterizer.heightMask) {
        const local166 = clamp(x + local14, _Rasterizer.startX, _Rasterizer.widthMask);
        const local174 = clamp(x - local14, _Rasterizer.startX, _Rasterizer.widthMask);
        if (local142 <= _Rasterizer.heightMask) {
          ArrayUtils.fillRange(
            _Rasterizer.pixels[local142],
            local174,
            local166,
            fillColor
          );
        }
        if (_Rasterizer.startY <= local138) {
          ArrayUtils.fillRange(
            _Rasterizer.pixels[local138],
            local174,
            local166,
            fillColor
          );
        }
      }
    }
  }
};
var RasterizerOperationShape = class {
  constructor(fillColor, outlineColor, outlineWidth) {
    this.fillColor = fillColor;
    this.outlineColor = outlineColor;
    this.outlineWidth = outlineWidth;
  }
};
var RasterizerOperationLine = class _RasterizerOperationLine extends RasterizerOperationShape {
  constructor(x0, y0, x1, y1, color, outlineWidth) {
    super(-1, color, outlineWidth);
    this.x0 = x0;
    this.y0 = y0;
    this.x1 = x1;
    this.y1 = y1;
  }
  static create(buffer) {
    const x0 = buffer.readShort();
    const y0 = buffer.readShort();
    const x1 = buffer.readShort();
    const y1 = buffer.readShort();
    const color = buffer.readMedium();
    const outlineWidth = buffer.readUnsignedByte();
    return new _RasterizerOperationLine(x0, y0, x1, y1, color, outlineWidth);
  }
  render(width, height) {
  }
  renderFill(width, height) {
  }
  renderOutline(width, height) {
    const x0 = this.x0 * width >> 12;
    const x1 = this.x1 * width >> 12;
    const y0 = this.y0 * height >> 12;
    const y1 = this.y1 * height >> 12;
    Rasterizer.rasterLine(x0, x1, y0, y1, this.outlineColor);
  }
};
var RasterizerOperationBezierCurve = class _RasterizerOperationBezierCurve extends RasterizerOperationShape {
  constructor(x0, y0, x1, y1, x2, y2, x3, y3, color, outlineWidth) {
    super(-1, color, outlineWidth);
    this.x0 = x0;
    this.y0 = y0;
    this.x1 = x1;
    this.y1 = y1;
    this.x2 = x2;
    this.y2 = y2;
    this.x3 = x3;
    this.y3 = y3;
  }
  static create(buffer) {
    const x0 = buffer.readShort();
    const y0 = buffer.readShort();
    const x1 = buffer.readShort();
    const y1 = buffer.readShort();
    const x2 = buffer.readShort();
    const y2 = buffer.readShort();
    const x3 = buffer.readShort();
    const y3 = buffer.readShort();
    const color = buffer.readMedium();
    const outlineWidth = buffer.readUnsignedByte();
    return new _RasterizerOperationBezierCurve(
      x0,
      y0,
      x1,
      y1,
      x2,
      y2,
      x3,
      y3,
      color,
      outlineWidth
    );
  }
  render(width, height) {
  }
  renderFill(width, height) {
  }
  renderOutline(width, height) {
    const x0 = width * this.x0 >> 12;
    const y0 = height * this.y0 >> 12;
    const x1 = width * this.x1 >> 12;
    const y1 = height * this.y1 >> 12;
    const x2 = width * this.x2 >> 12;
    const y2 = height * this.y2 >> 12;
    const x3 = width * this.x3 >> 12;
    const y3 = height * this.y3 >> 12;
    Rasterizer.rasterBezierCurve(x0, y0, x1, y1, x2, y2, x3, y3, this.outlineColor);
  }
};
var RasterizerOperationRectangle = class _RasterizerOperationRectangle extends RasterizerOperationShape {
  constructor(x0, y0, x1, y1, fillColor, outlineColor, outlineWidth) {
    super(fillColor, outlineColor, outlineWidth);
    this.x0 = x0;
    this.y0 = y0;
    this.x1 = x1;
    this.y1 = y1;
  }
  static create(buffer) {
    const x0 = buffer.readShort();
    const y0 = buffer.readShort();
    const x1 = buffer.readShort();
    const y1 = buffer.readShort();
    const fillColor = buffer.readMedium();
    const outlineColor = buffer.readMedium();
    const outlineWidth = buffer.readUnsignedByte();
    return new _RasterizerOperationRectangle(
      x0,
      y0,
      x1,
      y1,
      fillColor,
      outlineColor,
      outlineWidth
    );
  }
  render(width, height) {
    const x0 = this.x0 * width >> 12;
    const x1 = this.x1 * width >> 12;
    const y0 = this.y0 * height >> 12;
    const y1 = this.y1 * height >> 12;
    Rasterizer.rasterRectangle(
      x0,
      x1,
      y0,
      y1,
      this.fillColor,
      this.outlineColor,
      this.outlineWidth
    );
  }
  renderFill(width, height) {
    const x0 = this.x0 * width >> 12;
    const x1 = this.x1 * width >> 12;
    const y0 = this.y0 * height >> 12;
    const y1 = this.y1 * height >> 12;
    Rasterizer.rasterRectangleFill(x0, x1, y0, y1, this.fillColor);
  }
  renderOutline(width, height) {
    const x0 = this.x0 * width >> 12;
    const x1 = this.x1 * width >> 12;
    const y0 = this.y0 * height >> 12;
    const y1 = this.y1 * height >> 12;
    Rasterizer.rasterRectangleOutline(x0, x1, y0, y1, this.outlineColor, this.outlineWidth);
  }
};
var RasterizerOperationEllipse = class _RasterizerOperationEllipse extends RasterizerOperationShape {
  constructor(x, y, sizeX, sizeY, fillColor, outlineColor, outlineWidth) {
    super(fillColor, outlineColor, outlineWidth);
    this.x = x;
    this.y = y;
    this.sizeX = sizeX;
    this.sizeY = sizeY;
  }
  static create(buffer) {
    const x = buffer.readShort();
    const y = buffer.readShort();
    const sizeX = buffer.readShort();
    const sizeY = buffer.readShort();
    const fillColor = buffer.readMedium();
    const outlineColor = buffer.readMedium();
    const outlineWidth = buffer.readUnsignedByte();
    return new _RasterizerOperationEllipse(
      x,
      y,
      sizeX,
      sizeY,
      fillColor,
      outlineColor,
      outlineWidth
    );
  }
  render(width, height) {
    const x = this.x * width >> 12;
    const y = this.y * height >> 12;
    const sizeX = this.sizeX * width >> 12;
    const sizeY = this.sizeY * height >> 12;
    Rasterizer.rasterEllipse(
      x,
      y,
      sizeX,
      sizeY,
      this.fillColor,
      this.outlineColor,
      this.outlineWidth
    );
  }
  renderFill(width, height) {
    const x = this.x * width >> 12;
    const y = this.y * height >> 12;
    const sizeX = this.sizeX * width >> 12;
    const sizeY = this.sizeY * height >> 12;
    Rasterizer.rasterEllipseFill(x, y, sizeX, sizeY, this.fillColor);
  }
  renderOutline(width, height) {
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/SpriteSourceOperation.ts
var SpriteSourceOperation = class extends TextureOperation {
  constructor() {
    super(0, false);
    this.spriteId = -1;
    this.width = 0;
    this.height = 0;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.spriteId = buffer.readUnsignedShort();
    }
  }
  getSpriteId() {
    return this.spriteId;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty && this.loadSprite(textureGenerator) && this.pixels) {
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      let offset = this.width * (textureGenerator.height === this.height ? line : line * this.height / textureGenerator.height | 0);
      if (textureGenerator.width === this.width) {
        for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
          const value = this.pixels[offset++];
          outputB[pixel] = value << 4 & 4080;
          outputG[pixel] = value >> 4 & 4080;
          outputR[pixel] = (value & 16711680) >> 12;
        }
      } else {
        for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
          const i_9_ = this.width * pixel / textureGenerator.width | 0;
          const value = this.pixels[offset + i_9_];
          outputB[pixel] = value << 4 & 4080;
          outputG[pixel] = (value & 65280) >> 4;
          outputR[pixel] = value >> 12 & 4080;
        }
      }
    }
    return output;
  }
  loadSprite(textureGenerator) {
    if (this.pixels) {
      return true;
    }
    if (this.spriteId >= 0) {
      const sprite = textureGenerator.loadSprite(this.spriteId);
      sprite.normalize();
      this.pixels = sprite.getPixelsRgb();
      this.width = sprite.width;
      this.height = sprite.height;
      return true;
    }
    return false;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/SquareWaveformOperation.ts
var SquareWaveformOperation = class extends TextureOperation {
  constructor() {
    super(0, true);
    this.field0 = 10;
    this.field1 = 2048;
    this.field2 = 0;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.field0 = buffer.readUnsignedByte();
    } else if (field === 1) {
      this.field1 = buffer.readUnsignedShort();
    } else if (field === 2) {
      this.field2 = buffer.readUnsignedByte();
    }
  }
  init() {
    this.table0 = new Int32Array(this.field0 + 1);
    this.table1 = new Int32Array(this.field0 + 1);
    let i = 0;
    let j = 4096 / this.field0 | 0;
    let k = j * this.field1 >> 12;
    for (let l = 0; l < this.field0; l++) {
      this.table1[l] = i;
      this.table0[l] = i + k;
      i += j;
    }
    this.table1[this.field0] = 4096;
    this.table0[this.field0] = this.table0[0] + 4096;
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const verticalGradient = textureGenerator.verticalGradient[line];
      if (this.field2 === 0) {
        let value = 0;
        for (let i = 0; i < this.field0; i++) {
          if (verticalGradient >= this.table1[i] && verticalGradient < this.table1[i + 1]) {
            if (verticalGradient < this.table0[i]) {
              value = 4096;
            }
            break;
          }
        }
        output.fill(value, 0, textureGenerator.width);
      } else {
        for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
          let index = 0;
          let value = 0;
          const horizontalGradient = textureGenerator.horizontalGradient[pixel];
          switch (this.field2) {
            case 3:
              index = (horizontalGradient - verticalGradient >> 1) + 2048;
              break;
            case 2:
              index = (horizontalGradient - (4096 - verticalGradient) >> 1) + 2048;
              break;
            case 1:
              index = horizontalGradient;
              break;
          }
          for (let i = 0; i < this.field0; i++) {
            if (index >= this.table1[i] && index < this.table1[i + 1]) {
              if (index < this.table0[i]) {
                value = 4096;
              }
              break;
            }
          }
          output[pixel] = value;
        }
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/TextureSourceOperation.ts
var TextureSourceOperation = class extends TextureOperation {
  constructor() {
    super(0, false);
    this.textureId = -1;
    this.width = 0;
    this.height = 0;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.textureId = buffer.readUnsignedShort();
    }
  }
  initCaches(textureGenerator, width, height) {
    super.initCaches(textureGenerator, width, height);
    if (this.textureId >= 0) {
      const width2 = textureGenerator.textureLoader.isSmall(this.textureId) ? 64 : 128;
      this.pixels = textureGenerator.textureLoader.getPixelsRgb(
        this.textureId,
        width2,
        false,
        1
      );
      this.width = width2;
      this.height = width2;
    }
  }
  clearCaches() {
    super.clearCaches();
    this.pixels = void 0;
  }
  getTextureId() {
    return this.textureId;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      let start = (textureGenerator.height === this.height ? line : this.height * line / textureGenerator.height | 0) * this.width;
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      if (textureGenerator.width === this.width) {
        for (let x = 0; x < textureGenerator.width; x++) {
          const value = this.pixels[start++];
          outputB[x] = (value & 255) << 4;
          outputG[x] = (value & 65280) >> 4;
          outputR[x] = (value & 16711680) >> 12;
        }
      } else {
        for (let x = 0; x < textureGenerator.width; x++) {
          const idx = this.width * x / textureGenerator.width | 0;
          const value = this.pixels[idx + start];
          outputB[x] = (value & 255) << 4;
          outputG[x] = (value & 65280) >> 4;
          outputR[x] = (value & 16711680) >> 12;
        }
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/TilingOperation.ts
var TilingOperation = class extends TextureOperation {
  constructor() {
    super(1, false);
    this.tileCountH = 4;
    this.tileCountV = 4;
  }
  decode(field, buffer) {
    switch (field) {
      case 0:
        this.tileCountH = buffer.readUnsignedByte();
        break;
      case 1:
        this.tileCountV = buffer.readUnsignedByte();
        break;
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const tileW = textureGenerator.width / this.tileCountH | 0;
      const tileH = textureGenerator.height / this.tileCountV | 0;
      let input;
      if (tileH <= 0) {
        input = this.getMonochromeInput(textureGenerator, 0, 0);
      } else {
        const tY = line % tileH;
        input = this.getMonochromeInput(
          textureGenerator,
          0,
          textureGenerator.height * tY / tileH | 0
        );
      }
      for (let x = 0; x < textureGenerator.width; x++) {
        if (tileW <= 0) {
          output[x] = input[0];
        } else {
          const tX = x % tileW;
          output[x] = input[tX * textureGenerator.width / tileW | 0];
        }
      }
    }
    return output;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const tileW = textureGenerator.width / this.tileCountH | 0;
      const tileH = textureGenerator.height / this.tileCountV | 0;
      let input;
      if (tileH <= 0) {
        input = this.getColourInput(textureGenerator, 0, 0);
      } else {
        const tY = line % tileH;
        input = this.getColourInput(
          textureGenerator,
          0,
          textureGenerator.height * tY / tileH | 0
        );
      }
      const inputR = input[0];
      const inputG = input[1];
      const inputB = input[2];
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      for (let x = 0; x < textureGenerator.width; x++) {
        let inputX = 0;
        if (tileW > 0) {
          const tX = x % tileW;
          inputX = tX * textureGenerator.width / tileW | 0;
        }
        outputR[x] = inputR[inputX];
        outputG[x] = inputG[inputX];
        outputB[x] = inputB[inputX];
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/TilingSpriteOperation.ts
var TilingSpriteOperation = class extends SpriteSourceOperation {
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty && super.loadSprite(textureGenerator) && this.pixels) {
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      const startY = this.height * (line % this.height);
      for (let x = 0; x < textureGenerator.width; x++) {
        const rgb = this.pixels[startY + x % this.width];
        outputR[x] = rgb >> 12 & 4080;
        outputG[x] = rgb >> 4 & 4080;
        outputB[x] = (rgb & 255) << 4;
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/TrigWarpOperation.ts
var TrigWarpOperation = class extends TextureOperation {
  constructor() {
    super(3, false);
    this.hypotenuseMultiplier = 32768;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.hypotenuseMultiplier = buffer.readUnsignedShort() << 4;
    } else if (field === 1) {
      this.isMonochrome = buffer.readUnsignedByte() === 1;
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const inputA = this.getMonochromeInput(textureGenerator, 1, line);
      const inputB = this.getMonochromeInput(textureGenerator, 2, line);
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        const angle = inputA[pixel] >> 4 & 255;
        const hyp = inputB[pixel] * this.hypotenuseMultiplier >> 12;
        const cosine = TextureGenerator.COSINE[angle] * hyp >> 12;
        const sine = TextureGenerator.SINE[angle] * hyp >> 12;
        const nX = pixel + (cosine >> 12) & textureGenerator.widthMask;
        const nY = line + (sine >> 12) & textureGenerator.heightMask;
        const input = this.getMonochromeInput(textureGenerator, 0, nY);
        output[pixel] = input[nX];
      }
    }
    return output;
  }
  getColourOutput(textureGenerator, line) {
    if (!this.colourImageCache) {
      throw new Error("Colour image cache is not initialized");
    }
    const output = this.colourImageCache.get(line);
    if (this.colourImageCache.dirty) {
      const inputA = this.getMonochromeInput(textureGenerator, 1, line);
      const inputB = this.getMonochromeInput(textureGenerator, 2, line);
      const outputR = output[0];
      const outputG = output[1];
      const outputB = output[2];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        const angle = inputA[pixel] * 255 >> 12 & 255;
        const hyp = inputB[pixel] * this.hypotenuseMultiplier >> 12;
        const cosine = TextureGenerator.COSINE[angle] * hyp >> 12;
        const sine = TextureGenerator.SINE[angle] * hyp >> 12;
        const nX = pixel + (cosine >> 12) & textureGenerator.widthMask;
        const nY = line + (sine >> 12) & textureGenerator.heightMask;
        const input = this.getColourInput(textureGenerator, 0, nY);
        outputR[pixel] = input[0][nX];
        outputG[pixel] = input[1][nX];
        outputB[pixel] = input[2][nX];
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/VerticalGradientOperation.ts
var VerticalGradientOperation = class extends TextureOperation {
  constructor() {
    super(0, true);
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      output.fill(textureGenerator.verticalGradient[line], 0, textureGenerator.width);
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/VoronoiNoiseOperation.ts
var import_java_random5 = __toESM(require_lib());
var VoronoiNoiseOperation = class _VoronoiNoiseOperation extends TextureOperation {
  constructor() {
    super(0, true);
    this.rngSeed = 0;
    this.field2 = 2048;
    this.field3 = 2;
    this.field4 = 1;
    this.field5 = 5;
    this.field6 = 5;
    this.permutations = new Int8Array(512);
    this.randomNs = new Int16Array(512);
  }
  static {
    this.temp0 = 0;
  }
  static {
    this.temp1 = 0;
  }
  static {
    this.temp2 = 0;
  }
  static {
    this.temp3 = 0;
  }
  decode(field, buffer) {
    switch (field) {
      case 0:
        this.field5 = this.field6 = buffer.readUnsignedByte();
        break;
      case 1:
        this.rngSeed = buffer.readUnsignedByte();
        break;
      case 2:
        this.field2 = buffer.readUnsignedShort();
        break;
      case 3:
        this.field3 = buffer.readUnsignedByte();
        break;
      case 4:
        this.field4 = buffer.readUnsignedByte();
        break;
      case 5:
        this.field5 = buffer.readUnsignedByte();
        break;
      case 6:
        this.field6 = buffer.readUnsignedByte();
        break;
    }
  }
  init() {
    this.permutations = TextureGenerator.initPermutations(this.rngSeed);
    this.initRandomNumbers();
  }
  initRandomNumbers() {
    const random = new import_java_random5.default(this.rngSeed);
    this.randomNs = new Int16Array(512);
    if (this.field2 > 0) {
      for (let i = 0; i < 512; i++) {
        this.randomNs[i] = nextIntJagex(random, this.field2);
      }
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const i_1_ = 2048 + this.field6 * textureGenerator.verticalGradient[line];
      const i_2_ = i_1_ >> 12;
      const i_3_ = i_2_ + 1;
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        _VoronoiNoiseOperation.temp0 = 2147483647;
        _VoronoiNoiseOperation.temp1 = 2147483647;
        _VoronoiNoiseOperation.temp2 = 2147483647;
        _VoronoiNoiseOperation.temp3 = 2147483647;
        const i_5_ = this.field5 * textureGenerator.horizontalGradient[pixel] + 2048;
        const i_6_ = i_5_ >> 12;
        const i_7_ = i_6_ + 1;
        for (let i_8_ = i_2_ - 1; i_8_ <= i_3_; i_8_++) {
          const i_9_ = this.permutations[(i_8_ >= this.field6 ? i_8_ - this.field6 : i_8_) & 255] & 255;
          for (let i_10_ = i_6_ - 1; i_10_ <= i_7_; i_10_++) {
            let i_11_ = (this.permutations[(i_10_ >= this.field5 ? i_10_ - this.field5 : i_10_) + i_9_ & 255] & 255) * 2;
            let i_12_ = i_5_ - (this.randomNs[i_11_++] + (i_10_ << 12));
            let i_13_ = i_1_ - (this.randomNs[i_11_] + (i_8_ << 12));
            let i_14_;
            switch (this.field4) {
              case 1:
                i_14_ = i_12_ * i_12_ + i_13_ * i_13_ >> 12;
                break;
              case 2:
                i_14_ = (i_13_ < 0 ? -i_13_ : i_13_) + (i_12_ < 0 ? -i_12_ : i_12_);
                break;
              case 3:
                i_12_ = i_12_ < 0 ? -i_12_ : i_12_;
                i_13_ = i_13_ < 0 ? -i_13_ : i_13_;
                i_14_ = Math.max(i_12_, i_13_);
                break;
              case 4:
                i_12_ = Math.sqrt(Math.fround(i_12_ < 0 ? -i_12_ : i_12_) / 4096) * 4096 | 0;
                i_13_ = Math.sqrt(Math.fround(i_13_ < 0 ? -i_13_ : i_13_) / 4096) * 4096 | 0;
                i_14_ = i_13_ + i_12_;
                i_14_ = i_14_ * i_14_ >> 12;
                break;
              case 5:
                i_12_ *= i_12_;
                i_13_ *= i_13_;
                i_14_ = Math.sqrt(
                  Math.sqrt(Math.fround((i_12_ + i_13_) / 16777216))
                ) * 4096 | 0;
                break;
              default:
                i_14_ = Math.sqrt(
                  Math.fround((i_13_ * i_13_ + i_12_ * i_12_) / 16777216)
                ) * 4096 | 0;
                break;
            }
            if (i_14_ < _VoronoiNoiseOperation.temp3) {
              _VoronoiNoiseOperation.temp0 = _VoronoiNoiseOperation.temp1;
              _VoronoiNoiseOperation.temp1 = _VoronoiNoiseOperation.temp2;
              _VoronoiNoiseOperation.temp2 = _VoronoiNoiseOperation.temp3;
              _VoronoiNoiseOperation.temp3 = i_14_;
            } else if (i_14_ < _VoronoiNoiseOperation.temp2) {
              _VoronoiNoiseOperation.temp0 = _VoronoiNoiseOperation.temp1;
              _VoronoiNoiseOperation.temp1 = _VoronoiNoiseOperation.temp2;
              _VoronoiNoiseOperation.temp2 = i_14_;
            } else if (i_14_ < _VoronoiNoiseOperation.temp1) {
              _VoronoiNoiseOperation.temp0 = _VoronoiNoiseOperation.temp1;
              _VoronoiNoiseOperation.temp1 = i_14_;
            } else if (i_14_ < _VoronoiNoiseOperation.temp0) {
              _VoronoiNoiseOperation.temp0 = i_14_;
            }
          }
        }
        switch (this.field3) {
          case 0:
            output[pixel] = _VoronoiNoiseOperation.temp3;
            break;
          case 1:
            output[pixel] = _VoronoiNoiseOperation.temp2;
            break;
          case 2:
            output[pixel] = _VoronoiNoiseOperation.temp2 - _VoronoiNoiseOperation.temp3;
            break;
          case 3:
            output[pixel] = _VoronoiNoiseOperation.temp1;
            break;
          case 4:
            output[pixel] = _VoronoiNoiseOperation.temp0;
            break;
        }
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/WeaveOperation.ts
var WeaveOperation = class extends TextureOperation {
  constructor() {
    super(0, true);
    this.thickness = 585;
  }
  decode(field, buffer) {
    if (field === 0) {
      this.thickness = buffer.readUnsignedShort();
    }
  }
  getMonochromeOutput(textureGenerator, line) {
    if (!this.monochromeImageCache) {
      throw new Error("Monochrome image cache is not initialized");
    }
    const output = this.monochromeImageCache.get(line);
    if (this.monochromeImageCache.dirty) {
      const gradV = textureGenerator.verticalGradient[line];
      for (let pixel = 0; pixel < textureGenerator.width; pixel++) {
        const gradH = textureGenerator.horizontalGradient[pixel];
        if (gradH > this.thickness && 4096 - this.thickness > gradH && gradV > 2048 - this.thickness && gradV < this.thickness + 2048) {
          let v = 2048 - gradH;
          v = v < 0 ? -v : v;
          v <<= 12;
          v /= 2048 - this.thickness;
          output[pixel] = 4096 - v;
        } else if (2048 - this.thickness < gradH && 2048 + this.thickness > gradH) {
          let v = gradV - 2048;
          v = v < 0 ? -v : v;
          v -= this.thickness;
          v <<= 12;
          output[pixel] = v / (2048 - this.thickness);
        } else if (this.thickness > gradV || gradV > 4096 - this.thickness) {
          let v = gradH - 2048;
          v = v < 0 ? -v : v;
          v -= this.thickness;
          v <<= 12;
          output[pixel] = v / (2048 - this.thickness);
        } else if (gradH < this.thickness || 4096 - this.thickness < gradH) {
          let v = 2048 - gradV;
          v = v < 0 ? -v : v;
          v <<= 12;
          v /= 2048 - this.thickness;
          output[pixel] = 4096 - v;
        } else {
          output[pixel] = 0;
        }
      }
    }
    return output;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/operation/TextureOperationFactory.ts
var TextureOperationFactory = class _TextureOperationFactory {
  static instantiate(id) {
    switch (id) {
      case 0:
        return new ConstantMonochromeOperation();
      case 1:
        return new ConstantColourOperation();
      case 2:
        return new HorizontalGradientOperation();
      case 3:
        return new VerticalGradientOperation();
      case 4:
        return new BricksOperation();
      case 5:
        return new BlurOperation();
      case 6:
        return new ClampOperation();
      case 7:
        return new ArithmeticOperation();
      case 8:
        return new CurveOperation();
      case 9:
        return new MirrorOperation();
      case 10:
        return new GradientOperation();
      case 11:
        return new ColourStripOperation();
      case 12:
        return new DiagonalGradientOperation();
      case 13:
        return new PseudoRandomNoiseOperation();
      case 14:
        return new WeaveOperation();
      case 15:
        return new VoronoiNoiseOperation();
      case 16:
        return new HerringboneOperation();
      case 17:
        return new HslOperation();
      case 18:
        return new TilingSpriteOperation();
      case 19:
        return new TrigWarpOperation();
      case 20:
        return new TilingOperation();
      case 21:
        return new MixerOperation();
      case 22:
        return new InvertOperation();
      case 23:
        return new KaleidoscopeOperation();
      case 24:
        return new GrayScaleOperation();
      case 25:
        return new BrightnessOperation();
      case 26:
        return new BinaryOperation();
      case 27:
        return new SquareWaveformOperation();
      case 28:
        return new IrregularBricksOperation();
      case 29:
        return new RasterizerOperation();
      case 30:
        return new RangeOperation();
      case 31:
        return new MandelbrotOperation();
      case 32:
        return new EmbossOperation();
      case 33:
        return new ColorEdgeDetectorOperation();
      case 34:
        return new PerlinNoiseOperation();
      case 35:
        return new MonochromeEdgeDetectorOperation();
      case 36:
        return new TextureSourceOperation();
      case 37:
        return new Operation37();
      case 38:
        return new LineNoiseOperation();
      case 39:
        return new SpriteSourceOperation();
      default:
        throw new Error("Unknown texture operation: " + id);
    }
  }
  static create(buffer) {
    const id = buffer.readUnsignedByte();
    const type = buffer.readUnsignedByte();
    const operation = _TextureOperationFactory.instantiate(type);
    operation.id = id;
    operation.cacheSize = buffer.readUnsignedByte();
    const fieldCount = buffer.readUnsignedByte();
    for (let i = 0; i < fieldCount; i++) {
      const field = buffer.readUnsignedByte();
      operation.decode(field, buffer);
    }
    operation.init();
    return operation;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/procedural/ProceduralTexture.ts
var ProceduralTexture = class {
  constructor(buffer, hasModOperation) {
    const operationCount = buffer.readUnsignedByte();
    let spriteDepCount = 0;
    let textureDepCount = 0;
    const inputConnections = new Array(operationCount);
    this.operations = new Array(operationCount);
    for (let op = 0; op < operationCount; op++) {
      const operation = TextureOperationFactory.create(buffer);
      if (operation.getSpriteId() >= 0) {
        spriteDepCount++;
      }
      if (operation.getTextureId() >= 0) {
        textureDepCount++;
      }
      const inputCount = operation.inputs.length;
      inputConnections[op] = new Array(inputCount);
      for (let i = 0; i < inputCount; i++) {
        inputConnections[op][i] = buffer.readUnsignedByte();
      }
      this.operations[op] = operation;
    }
    this.spriteDependencies = new Array(spriteDepCount);
    this.textureDependencies = new Array(textureDepCount);
    let spriteDepIndex = 0;
    let textureDepIndex = 0;
    for (let op = 0; op < operationCount; op++) {
      const operation = this.operations[op];
      const inputCount = operation.inputs.length;
      for (let i = 0; i < inputCount; i++) {
        operation.inputs[i] = this.operations[inputConnections[op][i]];
      }
      const spriteId = operation.getSpriteId();
      const textureId = operation.getTextureId();
      if (spriteId >= 0) {
        this.spriteDependencies[spriteDepIndex++] = spriteId;
      }
      if (textureId >= 0) {
        this.textureDependencies[textureDepIndex++] = textureId;
      }
      delete inputConnections[op];
    }
    this.colourOperation = this.operations[buffer.readUnsignedByte()];
    this.alphaOperation = this.operations[buffer.readUnsignedByte()];
    if (hasModOperation) {
      this.monochromeOperation = this.operations[buffer.readUnsignedByte()];
    }
  }
  getPixelsRgb(textureGenerator, width, height, flipH, flipV, brightness) {
    if (textureGenerator.debug) {
      console.log("getPixelsRgb", this);
    }
    for (const operation of this.operations) {
      operation.initCaches(textureGenerator, width, height);
    }
    textureGenerator.initBrightness(brightness);
    textureGenerator.init(width, height);
    const pixels = new Int32Array(width * height);
    let srcInc;
    let srcEnd;
    let srcStart;
    if (flipH) {
      srcInc = -1;
      srcEnd = -1;
      srcStart = width - 1;
    } else {
      srcStart = 0;
      srcEnd = width;
      srcInc = 1;
    }
    let dstIdx = 0;
    for (let line = 0; line < height; line++) {
      if (flipV) {
        dstIdx = line;
      }
      let pixelsR;
      let pixelsG;
      let pixelsB;
      if (this.colourOperation.isMonochrome) {
        const output = this.colourOperation.getMonochromeOutput(textureGenerator, line);
        pixelsR = output;
        pixelsG = output;
        pixelsB = output;
      } else {
        const output = this.colourOperation.getColourOutput(textureGenerator, line);
        pixelsR = output[0];
        pixelsG = output[1];
        pixelsB = output[2];
      }
      for (let srcIdx = srcStart; srcIdx !== srcEnd; srcIdx += srcInc) {
        let r = pixelsR[srcIdx] >> 4;
        if (r > 255) {
          r = 255;
        }
        if (r < 0) {
          r = 0;
        }
        let g = pixelsG[srcIdx] >> 4;
        if (g > 255) {
          g = 255;
        }
        if (g < 0) {
          g = 0;
        }
        let b = pixelsB[srcIdx] >> 4;
        if (b > 255) {
          b = 255;
        }
        if (b < 0) {
          b = 0;
        }
        r = textureGenerator.brightnessTable[r];
        g = textureGenerator.brightnessTable[g];
        b = textureGenerator.brightnessTable[b];
        let rgb = r << 16 | g << 8 | b;
        if (rgb !== 0) {
          rgb |= 4278190080;
        } else {
          textureGenerator.isTransparent = true;
        }
        pixels[dstIdx++] = rgb;
        if (flipV) {
          dstIdx += width - 1;
        }
      }
    }
    for (const operation of this.operations) {
      operation.clearCaches();
    }
    return pixels;
  }
  getPixelsArgb(textureGenerator, width, height, flipH, flipV, brightness) {
    for (const operation of this.operations) {
      operation.initCaches(textureGenerator, width, height);
    }
    textureGenerator.initBrightness(brightness);
    textureGenerator.init(width, height);
    const pixels = new Int32Array(width * height);
    let srcInc;
    let srcEnd;
    let srcStart;
    if (flipH) {
      srcInc = -1;
      srcEnd = -1;
      srcStart = width - 1;
    } else {
      srcStart = 0;
      srcEnd = width;
      srcInc = 1;
    }
    let dstIdx = 0;
    for (let line = 0; line < height; line++) {
      if (flipV) {
        dstIdx = line;
      }
      let pixelsR;
      let pixelsG;
      let pixelsB;
      let pixelsA;
      if (this.colourOperation.isMonochrome) {
        const output = this.colourOperation.getMonochromeOutput(textureGenerator, line);
        pixelsR = output;
        pixelsG = output;
        pixelsB = output;
      } else {
        const output = this.colourOperation.getColourOutput(textureGenerator, line);
        pixelsR = output[0];
        pixelsG = output[1];
        pixelsB = output[2];
      }
      if (this.alphaOperation) {
        if (this.alphaOperation.isMonochrome) {
          pixelsA = this.alphaOperation.getMonochromeOutput(textureGenerator, line);
        } else {
          pixelsA = this.alphaOperation.getColourOutput(textureGenerator, line)[0];
        }
      }
      for (let srcIdx = srcStart; srcIdx !== srcEnd; srcIdx += srcInc) {
        let r = pixelsR[srcIdx] >> 4;
        if (r > 255) {
          r = 255;
        }
        if (r < 0) {
          r = 0;
        }
        let g = pixelsG[srcIdx] >> 4;
        if (g > 255) {
          g = 255;
        }
        if (g < 0) {
          g = 0;
        }
        let b = pixelsB[srcIdx] >> 4;
        if (b > 255) {
          b = 255;
        }
        if (b < 0) {
          b = 0;
        }
        r = textureGenerator.brightnessTable[r];
        g = textureGenerator.brightnessTable[g];
        b = textureGenerator.brightnessTable[b];
        let a = 0;
        if (r !== 0 || g !== 0 || b !== 0) {
          if (pixelsA) {
            a = clamp(pixelsA[srcIdx] >> 4, 0, 255);
          } else {
            a = 255;
          }
        }
        if (a !== 255) {
          textureGenerator.isTransparent = true;
        }
        const argb = a << 24 | r << 16 | g << 8 | b;
        pixels[dstIdx++] = argb;
        if (flipV) {
          dstIdx += width - 1;
        }
      }
    }
    if (textureGenerator.debug) {
      console.log("getPixelsArgb", this);
    }
    for (const operation of this.operations) {
      operation.clearCaches();
    }
    return pixels;
  }
};

// {TSPS_CHECKOUT}/client/rs/texture/ProceduralTextureLoader.ts
var ProceduralTextureLoader = class _ProceduralTextureLoader {
  constructor(isRunetek5, hasModOperation, textureIndex, spriteIndex, textureIds, materials) {
    this.isRunetek5 = isRunetek5;
    this.hasModOperation = hasModOperation;
    this.textureIndex = textureIndex;
    this.spriteIndex = spriteIndex;
    this.textureIds = textureIds;
    this.materials = materials;
    this.textures = /* @__PURE__ */ new Map();
    this.transparentTextureMap = /* @__PURE__ */ new Map();
    this.textureGenerator = new TextureGenerator(spriteIndex, this);
  }
  static load(revision, materialsIndex, textureIndex, spriteIndex) {
    const hasModOperation = revision >= 537;
    const hasCombineModeAndShaderParam2 = revision >= 582;
    const hasAlphaBlending = revision >= 629;
    const materialsFile = materialsIndex.getFile(0, 0);
    if (!materialsFile) {
      throw new Error("Materials file not found");
    }
    const buffer = materialsFile.getDataAsBuffer();
    const count = buffer.readUnsignedShort();
    const materials = new Array(count);
    for (let i = 0; i < count; i++) {
      const exists = buffer.readUnsignedByte() === 1;
      if (exists) {
        materials[i] = new ProcTextureMaterial(i);
      }
    }
    for (let i = 0; i < count; i++) {
      const material = materials[i];
      if (material) {
        material.valid = buffer.readUnsignedByte() === 1;
      }
    }
    if (!hasAlphaBlending) {
      for (let i = 0; i < count; i++) {
        const material = materials[i];
        if (material) {
          material.alpha = buffer.readUnsignedByte() === 1;
        }
      }
    }
    for (let i = 0; i < count; i++) {
      const material = materials[i];
      if (material) {
        material.small = buffer.readUnsignedByte() === 1;
      }
    }
    for (let i = 0; i < count; i++) {
      const material = materials[i];
      if (material) {
        material.disabled = buffer.readUnsignedByte() === 1;
      }
    }
    for (let i = 0; i < count; i++) {
      const material = materials[i];
      if (material) {
        material.brightness = buffer.readByte();
      }
    }
    for (let i = 0; i < count; i++) {
      const material = materials[i];
      if (material) {
        material.blanch = buffer.readByte();
      }
    }
    for (let i = 0; i < count; i++) {
      const material = materials[i];
      if (material) {
        material.shaderId = buffer.readByte();
      }
    }
    for (let i = 0; i < count; i++) {
      const material = materials[i];
      if (material) {
        material.shaderParam = buffer.readByte();
      }
    }
    for (let i = 0; i < count; i++) {
      const material = materials[i];
      if (material) {
        material.averageHsl = buffer.readUnsignedShort();
      }
    }
    if (buffer.remaining > 0) {
      for (let i = 0; i < count; i++) {
        const material = materials[i];
        if (material) {
          material.animU = buffer.readByte();
        }
      }
      for (let i = 0; i < count; i++) {
        const material = materials[i];
        if (material) {
          material.animV = buffer.readByte();
        }
      }
      for (let i = 0; i < count; i++) {
        const material = materials[i];
        if (material) {
          buffer.readByte();
        }
      }
      for (let i = 0; i < count; i++) {
        const material = materials[i];
        if (material) {
          material.flipV = buffer.readUnsignedByte() === 1;
        }
      }
      for (let i = 0; i < count; i++) {
        const material = materials[i];
        if (material) {
          material.mipmap = buffer.readByte();
        }
      }
      for (let i = 0; i < count; i++) {
        const material = materials[i];
        if (material) {
          material.repeatS = buffer.readUnsignedByte() === 1;
        }
      }
      for (let i = 0; i < count; i++) {
        const material = materials[i];
        if (material) {
          material.repeatT = buffer.readUnsignedByte() === 1;
        }
      }
      for (let i = 0; i < count; i++) {
        const material = materials[i];
        if (material) {
          material.floatTexture = buffer.readUnsignedByte() === 1;
        }
      }
      if (hasCombineModeAndShaderParam2) {
        for (let i = 0; i < count; i++) {
          const material = materials[i];
          if (material) {
            material.combineMode = buffer.readUnsignedByte();
          }
        }
        for (let i = 0; i < count; i++) {
          const material = materials[i];
          if (material) {
            material.shaderParam2 = buffer.readInt();
          }
        }
      }
      if (hasAlphaBlending) {
        for (let i = 0; i < count; i++) {
          const material = materials[i];
          if (material) {
            material.alphaMode = buffer.readUnsignedByte();
          }
        }
      }
    }
    const textureIds = Array.from(textureIndex.getArchiveIds());
    return new _ProceduralTextureLoader(
      revision >= 555,
      hasModOperation,
      textureIndex,
      spriteIndex,
      textureIds,
      materials
    );
  }
  getTexture(id) {
    const cached = this.textures.get(id);
    if (cached) {
      return cached;
    }
    const textureFile = this.textureIndex.getFileSmart(id);
    if (!textureFile) {
      return void 0;
    }
    const buffer = textureFile.getDataAsBuffer();
    const texture = new ProceduralTextureDefinition(id, buffer, this.hasModOperation);
    this.textures.set(id, texture);
    return texture;
  }
  getTextureIds() {
    return this.textureIds;
  }
  getTextureIndex(id) {
    return id;
  }
  isSd(id) {
    return this.materials[id]?.valid ?? false;
  }
  isSmall(id) {
    return this.materials[id]?.small ?? false;
  }
  isTransparent(id) {
    if (!this.transparentTextureMap.has(id)) {
      try {
        if (!this.getTexture(id)) {
          return false;
        }
        this.getPixelsArgb(id, 128, false, 1);
      } catch (e) {
        console.error("Error loading texture", e);
        this.transparentTextureMap.set(id, false);
      }
    }
    return this.transparentTextureMap.get(id) ?? false;
  }
  getAverageHsl(id) {
    return this.materials[id]?.averageHsl ?? 0;
  }
  getAnimationUv(id) {
    const texture = this.getTexture(id);
    if (!texture) {
      return [0, 0];
    }
    return [texture.animU, texture.animV];
  }
  getMaterial(id) {
    const texture = this.getTexture(id);
    const material = this.materials[id];
    if (!material) {
      return {
        animU: 0,
        animV: 0,
        alphaCutOff: 0.1,
        frameCount: 1,
        animSpeed: 0
      };
    }
    let animU = material.animU;
    let animV = material.animV;
    if (!this.isRunetek5) {
      const textureDef = this.getTexture(id);
      if (textureDef) {
        animU = textureDef.animU;
        animV = textureDef.animV;
      }
    }
    let alphaCutOff = 0.9;
    if (animU !== 0 || animV !== 0 || material.alphaMode === 2) {
      alphaCutOff = 0.01;
    }
    return {
      animU,
      animV,
      alphaCutOff,
      frameCount: 1,
      animSpeed: Math.max(Math.abs(animU), Math.abs(animV))
    };
  }
  getFrameCount(_id) {
    return 1;
  }
  getPixelsRgb(id, size, flipH, brightness, _frame) {
    const texture = this.getTexture(id);
    if (!texture) {
      throw new Error("Texture not found: " + id);
    }
    const pixels = texture.proceduralTexture.getPixelsRgb(
      this.textureGenerator,
      size,
      size,
      flipH,
      texture.flipV,
      brightness
    );
    this.transparentTextureMap.set(id, this.textureGenerator.isTransparent);
    return pixels;
  }
  getPixelsArgb(id, size, flipH, brightness, frame) {
    const texture = this.getTexture(id);
    if (!texture) {
      throw new Error("Texture not found: " + id);
    }
    const pixels = texture.proceduralTexture.getPixelsArgb(
      this.textureGenerator,
      size,
      size,
      flipH,
      texture.flipV,
      brightness
    );
    this.transparentTextureMap.set(id, this.textureGenerator.isTransparent);
    return pixels;
  }
};
var ProcTextureMaterial = class {
  constructor(id) {
    this.id = id;
    this.valid = false;
    this.alpha = false;
    this.small = false;
    this.disabled = false;
    this.brightness = 0;
    this.blanch = 0;
    this.shaderId = 0;
    this.shaderParam = 0;
    this.averageHsl = 0;
    this.animU = 0;
    this.animV = 0;
    this.flipV = false;
    this.mipmap = 0;
    this.repeatS = false;
    this.repeatT = false;
    this.floatTexture = false;
    this.combineMode = 0;
    this.shaderParam2 = 0;
    this.alphaMode = 0;
  }
};
var ProceduralTextureDefinition = class {
  constructor(id, buffer, hasModOperation) {
    this.bool1 = false;
    this.flipV = false;
    this.repeatS = false;
    this.repeatT = false;
    this.animU = 0;
    this.animV = 0;
    this.id = id;
    this.proceduralTexture = new ProceduralTexture(buffer, hasModOperation);
    this.bool1 = buffer.readUnsignedByte() === 1;
    this.flipV = buffer.readUnsignedByte() === 1;
    this.repeatS = buffer.readUnsignedByte() === 1;
    this.repeatT = buffer.readUnsignedByte() === 1;
    const combineMode = buffer.readUnsignedByte() & 3;
    this.animU = buffer.readByte();
    this.animV = buffer.readByte();
    if (combineMode === 1) {
      this.combineMode = 2 /* ADD */;
    } else if (combineMode === 2) {
      this.combineMode = 3 /* SUBTRACT */;
    } else if (combineMode === 3) {
      this.combineMode = 4 /* ADD_SIGNED */;
    } else {
      this.combineMode = 0 /* MODULATE */;
    }
  }
};

// {ROOT}/studies/tools-hd-era/client_bridge.ts
init_TextureMapper();
if (process.argv[2] === "decode" || process.argv[2] === "rawuv") {
  const { ModelData: ModelData2 } = (init_ModelData(), __toCommonJS(ModelData_exports));
  const m = ModelData2.decode(Int8Array.from(import_node_fs.default.readFileSync(0)));
  if (process.argv[2] === "rawuv") {
    const result = computeTextureCoords({ isSd: () => true }, m);
    process.stdout.write(Buffer.from(result.buffer));
  } else {
    const rows = (a, b, c) => Array.from(a, (x, i) => [x, b[i], c[i]]);
    process.stdout.write(JSON.stringify({ vertices: rows(m.verticesX, m.verticesY, m.verticesZ), faces: rows(m.indices1, m.indices2, m.indices3), colors: Array.from(m.faceColors, (x) => x & 65535) }));
  }
} else if (process.argv[2] === "uv") {
  const m = JSON.parse(import_node_fs.default.readFileSync(0, "utf8"));
  const uv = computeTextureCoords({ isSd: () => true }, m);
  process.stdout.write(Buffer.from(uv.buffer));
} else {
  const root = process.argv[2], rev = Number(process.argv[3]);
  const inputs = JSON.parse(import_node_fs.default.readFileSync(import_node_path.default.join(root, "inputs.json"), "utf8"));
  const index = (records) => {
    function file(id) {
      if (!(id in records)) return void 0;
      const data = Int8Array.from(Buffer.from(records[id], "base64"));
      return { data, getDataAsBuffer: () => new ByteBuffer(data) };
    }
    return { getFileSmart: file, getArchiveIds: () => Object.keys(records).map(Number), getFile: (g, f) => f === 0 ? file(g) : void 0 };
  };
  const loader = ProceduralTextureLoader.load(rev, index(inputs.materials), index(inputs.textures), index(inputs.sprites));
  const errors = {}, meta = {};
  for (const id of loader.getTextureIds()) {
    try {
      const t = loader.getTexture(id);
      const pixels = loader.getPixelsArgb(id, 128, false, 1);
      const rgba = Buffer.alloc(pixels.length * 4);
      for (let j = 0; j < pixels.length; j++) {
        rgba[j * 4] = pixels[j] >>> 16;
        rgba[j * 4 + 1] = pixels[j] >>> 8;
        rgba[j * 4 + 2] = pixels[j];
        rgba[j * 4 + 3] = pixels[j] >>> 24;
      }
      import_node_fs.default.writeFileSync(import_node_path.default.join(root, id + ".rgba"), rgba);
      meta[id] = { material: loader.materials[id], recipe: { sprites: t.proceduralTexture.spriteDependencies, textures: t.proceduralTexture.textureDependencies, operations: t.proceduralTexture.operations.map((op) => op.constructor.name) } };
    } catch (e) {
      errors[id] = String(e);
    }
  }
  import_node_fs.default.writeFileSync(import_node_path.default.join(root, "meta.json"), JSON.stringify(meta));
  import_node_fs.default.writeFileSync(import_node_path.default.join(root, "errors.json"), JSON.stringify(errors));
}

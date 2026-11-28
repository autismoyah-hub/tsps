var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __commonJS = (cb, mod) => function __require() {
  try {
    return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
  } catch (e) {
    throw mod = 0, e;
  }
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

// ../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/common.js
var require_common = __commonJS({
  "../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/common.js"(exports2) {
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

// ../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/mat2.js
var require_mat2 = __commonJS({
  "../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/mat2.js"(exports2) {
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

// ../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/mat2d.js
var require_mat2d = __commonJS({
  "../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/mat2d.js"(exports2) {
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

// ../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/mat3.js
var require_mat3 = __commonJS({
  "../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/mat3.js"(exports2) {
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

// ../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/mat4.js
var require_mat4 = __commonJS({
  "../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/mat4.js"(exports2) {
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

// ../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/vec3.js
var require_vec3 = __commonJS({
  "../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/vec3.js"(exports2) {
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

// ../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/vec4.js
var require_vec4 = __commonJS({
  "../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/vec4.js"(exports2) {
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

// ../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/quat.js
var require_quat = __commonJS({
  "../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/quat.js"(exports2) {
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
    var vec33 = _interopRequireWildcard(require_vec3());
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
      var tmpvec3 = vec33.create();
      var xUnitVec3 = vec33.fromValues(1, 0, 0);
      var yUnitVec3 = vec33.fromValues(0, 1, 0);
      return function(out, a, b) {
        var dot2 = vec33.dot(a, b);
        if (dot2 < -0.999999) {
          vec33.cross(tmpvec3, xUnitVec3, a);
          if (vec33.len(tmpvec3) < 1e-6) vec33.cross(tmpvec3, yUnitVec3, a);
          vec33.normalize(tmpvec3, tmpvec3);
          setAxisAngle(out, tmpvec3, Math.PI);
          return out;
        } else if (dot2 > 0.999999) {
          out[0] = 0;
          out[1] = 0;
          out[2] = 0;
          out[3] = 1;
          return out;
        } else {
          vec33.cross(tmpvec3, a, b);
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

// ../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/quat2.js
var require_quat2 = __commonJS({
  "../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/quat2.js"(exports2) {
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
    var mat44 = _interopRequireWildcard(require_mat4());
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
      mat44.getRotation(outer, a);
      var t = new glMatrix.ARRAY_TYPE(3);
      mat44.getTranslation(t, a);
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

// ../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/vec2.js
var require_vec2 = __commonJS({
  "../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/vec2.js"(exports2) {
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

// ../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/index.js
var require_cjs = __commonJS({
  "../../../tsps-work/tsps/client/node_modules/gl-matrix/cjs/index.js"(exports2) {
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
    var mat44 = _interopRequireWildcard(require_mat4());
    exports2.mat4 = mat44;
    var quat = _interopRequireWildcard(require_quat());
    exports2.quat = quat;
    var quat2 = _interopRequireWildcard(require_quat2());
    exports2.quat2 = quat2;
    var vec2 = _interopRequireWildcard(require_vec2());
    exports2.vec2 = vec2;
    var vec33 = _interopRequireWildcard(require_vec3());
    exports2.vec3 = vec33;
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

// pose.ts
var import_node_fs = __toESM(require("node:fs"));

// ../../../tsps-work/tsps/client/rs/MathConstants.ts
var TAU = Math.PI * 2;
var RS_TO_RADIANS = TAU / 2048;
var RS_TO_DEGREES = RS_TO_RADIANS * 180 / Math.PI;
var DEGREES_TO_RADIANS = Math.PI / 180;
function initBitMasks() {
  const masks = new Int32Array(32);
  let value = 2;
  for (let i = 0; i < 32; i++) {
    masks[i] = value - 1;
    value += value;
  }
  return masks;
}
var BIT_MASKS = initBitMasks();
var SINE = new Int32Array(2048);
var COSINE = new Int32Array(2048);
var CIRCULAR_ANGLE = 2048;
var ANGULAR_RATIO = 360 / CIRCULAR_ANGLE;
var ANGULAR_RATIO_RADIANS = ANGULAR_RATIO * DEGREES_TO_RADIANS;
for (let i = 0; i < 2048; i++) {
  SINE[i] = 65536 * Math.sin(i * ANGULAR_RATIO_RADIANS) | 0;
  COSINE[i] = 65536 * Math.cos(i * ANGULAR_RATIO_RADIANS) | 0;
}
var SINE_LARGE = new Int32Array(16384);
var COSINE_LARGE = new Int32Array(16384);
var d = 3834951969714103e-19;
for (let i = 0; i < 16384; i++) {
  SINE_LARGE[i] = 16384 * Math.sin(i * d);
  COSINE_LARGE[i] = 16384 * Math.cos(i * d);
}

// ../../../tsps-work/tsps/client/common/utils/FloatUtil.ts
var FloatUtil = class _FloatUtil {
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

// ../../../tsps-work/tsps/client/rs/io/ByteBuffer.ts
var ByteBuffer = class {
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
    const bytes2 = this._data.subarray(this.offset, this.offset + amount);
    this.offset += amount;
    return bytes2;
  }
  readUnsignedBytes(amount) {
    const bytes2 = new Uint8Array(this._data.buffer, this._data.byteOffset, this._data.byteLength).subarray(this.offset, this.offset + amount);
    this.offset += amount;
    return bytes2;
  }
  writeBytes(bytes2) {
    this._data.set(bytes2, this.offset);
    this.offset += bytes2.length;
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

// ../../../tsps-work/tsps/client/rs/scene/entity/Entity.ts
var Entity = class {
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

// ../../../tsps-work/tsps/client/rs/util/ColorUtil.ts
var BRIGHTNESS_LOW = 0.8;
var BRIGHTNESS_MINIMAP = 0.949999988079071;
var HUE_OFFSET = 78125e-7;
var SATURATION_OFFSET = 0.0625;
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
var HSL_RGB_MAP = buildPalette(BRIGHTNESS_LOW, 0, 512);
var MINIMAP_HSL_RGB_MAP = buildMinimapPalette();
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
var RGB15_TO_HSL_MAP = buildRgb15ToHslMap();
function buildRgb15ToHslMap() {
  const map = new Int32Array(32768);
  for (let i = 0; i < 32768; i++) {
    map[i] = rgb15ToHsl(i);
  }
  return map;
}

// ../../../tsps-work/tsps/client/rs/model/FaceNormal.ts
var FaceNormal = class {
  constructor(x, y, z) {
    this.x = x;
    this.y = y;
    this.z = z;
  }
};

// ../../../tsps-work/tsps/client/rs/model/Model.ts
var import_gl_matrix = __toESM(require_cjs());
var scaleVector = import_gl_matrix.vec3.create();
var Model = class _Model extends Entity {
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

// ../../../tsps-work/tsps/client/rs/model/TextureMapper.ts
var uvTemp = new Float32Array(2);
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
var TextureScales = class {
  constructor(centerXs, centerYs, centerZs, fs2) {
    this.centerXs = centerXs;
    this.centerYs = centerYs;
    this.centerZs = centerZs;
    this.fs = fs2;
  }
};
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

// ../../../tsps-work/tsps/client/rs/model/VertexNormal.ts
var VertexNormal = class _VertexNormal {
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

// ../../../tsps-work/tsps/client/rs/model/ModelData.ts
var ModelData = class _ModelData extends Entity {
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

// ../../../tsps-work/tsps/client/rs/model/skeletal/SkeletalBone.ts
var import_gl_matrix3 = __toESM(require_cjs());

// ../../../tsps-work/tsps/client/rs/model/skeletal/MatrixPool.ts
var import_gl_matrix2 = __toESM(require_cjs());
var MatrixPool = class _MatrixPool {
  static {
    this.IDENTITY = import_gl_matrix2.mat4.create();
  }
  static init(size) {
    _MatrixPool.matrixIndex = 0;
    _MatrixPool.matrixLimit = size;
    _MatrixPool.matrixPool = new Array(size);
  }
  static get() {
    if (_MatrixPool.matrixIndex === 0) {
      return import_gl_matrix2.mat4.create();
    } else {
      import_gl_matrix2.mat4.identity(_MatrixPool.matrixPool[--_MatrixPool.matrixIndex]);
      return _MatrixPool.matrixPool[_MatrixPool.matrixIndex];
    }
  }
  static release(m) {
    if (_MatrixPool.matrixIndex < _MatrixPool.matrixLimit - 1) {
      _MatrixPool.matrixPool[_MatrixPool.matrixIndex++] = m;
    }
  }
};
MatrixPool.init(100);

// ../../../tsps-work/tsps/client/rs/model/skeletal/SkeletalBone.ts
var SkeletalBone = class _SkeletalBone {
  constructor(poseCount, buffer, matrixCompact) {
    this.animMatrix = import_gl_matrix3.mat4.create();
    this.updateAnimModelMatrix = false;
    this.updateFinalMatrix = false;
    this.animModelMatrix = import_gl_matrix3.mat4.create();
    this.finalMatrix = import_gl_matrix3.mat4.create();
    this.parentId = buffer.readShort();
    this.localMatrices = new Array(poseCount);
    this.modelMatrices = new Array(poseCount);
    this.invertedModelMatrices = new Array(poseCount);
    const unused = new Array(poseCount);
    for (let i = 0; i < poseCount; i++) {
      this.localMatrices[i] = _SkeletalBone.readMat4(buffer, matrixCompact);
      unused[i] = new Array(3);
      unused[i][0] = buffer.readFloat();
      unused[i][1] = buffer.readFloat();
      unused[i][2] = buffer.readFloat();
    }
    this.extractTransformations();
  }
  static readMat4(buffer, compact) {
    if (compact) {
      throw new Error("Not implemented");
    } else {
      const m = new Float32Array(16);
      for (let i = 0; i < 16; i++) {
        m[i] = buffer.readFloat();
      }
      return m;
    }
  }
  static getRotation(out, m) {
    out[0] = -Math.asin(m[6]);
    out[1] = 0;
    out[2] = 0;
    const cosRotationX = Math.cos(out[0]);
    if (Math.abs(cosRotationX) > 5e-3) {
      out[1] = Math.atan2(m[2], m[10]);
      out[2] = Math.atan2(m[4], m[5]);
    } else {
      const sinRotationY = m[1];
      const cosRotationY = m[0];
      if (m[6] < 0) {
        out[1] = Math.atan2(sinRotationY, cosRotationY);
      } else {
        out[1] = -Math.atan2(sinRotationY, cosRotationY);
      }
      out[2] = 0;
    }
    return out;
  }
  extractTransformations() {
    const poseCount = this.localMatrices.length;
    this.rotations = new Array(poseCount);
    this.translations = new Array(poseCount);
    this.scalings = new Array(poseCount);
    const invertedLocalMatrix = MatrixPool.get();
    for (let i = 0; i < poseCount; i++) {
      const localMatrix = this.getLocalMatrix(i);
      import_gl_matrix3.mat4.invert(invertedLocalMatrix, localMatrix);
      this.rotations[i] = import_gl_matrix3.vec3.create();
      this.translations[i] = import_gl_matrix3.vec3.create();
      this.scalings[i] = import_gl_matrix3.vec3.create();
      _SkeletalBone.getRotation(this.rotations[i], invertedLocalMatrix);
      import_gl_matrix3.mat4.getTranslation(this.translations[i], localMatrix);
      import_gl_matrix3.mat4.getScaling(this.scalings[i], localMatrix);
    }
    MatrixPool.release(invertedLocalMatrix);
  }
  getLocalMatrix(poseId) {
    return this.localMatrices[poseId];
  }
  getModelMatrix(poseId) {
    if (this.modelMatrices[poseId] === void 0) {
      const modelMatrix = import_gl_matrix3.mat4.create();
      if (this.parent) {
        import_gl_matrix3.mat4.mul(
          modelMatrix,
          this.parent.getModelMatrix(poseId),
          this.getLocalMatrix(poseId)
        );
      } else {
        import_gl_matrix3.mat4.copy(modelMatrix, this.getLocalMatrix(poseId));
      }
      this.modelMatrices[poseId] = modelMatrix;
    }
    return this.modelMatrices[poseId];
  }
  getInvertedModelMatrix(poseId) {
    if (this.invertedModelMatrices[poseId] === void 0) {
      const inverted = import_gl_matrix3.mat4.invert(import_gl_matrix3.mat4.create(), this.getModelMatrix(poseId));
      this.invertedModelMatrices[poseId] = inverted || import_gl_matrix3.mat4.create();
    }
    return this.invertedModelMatrices[poseId];
  }
  setAnimMatrix(animMatrix) {
    import_gl_matrix3.mat4.copy(this.animMatrix, animMatrix);
    this.updateAnimModelMatrix = true;
    this.updateFinalMatrix = true;
  }
  getAnimMatrix() {
    return this.animMatrix;
  }
  getAnimModelMatrix() {
    if (this.updateAnimModelMatrix) {
      this.updateAnimModelMatrix = false;
      if (this.parent) {
        import_gl_matrix3.mat4.mul(
          this.animModelMatrix,
          this.parent.getAnimModelMatrix(),
          this.getAnimMatrix()
        );
      } else {
        import_gl_matrix3.mat4.copy(this.animModelMatrix, this.getAnimMatrix());
      }
    }
    return this.animModelMatrix;
  }
  getFinalMatrix(poseId) {
    if (this.updateFinalMatrix) {
      this.updateFinalMatrix = false;
      import_gl_matrix3.mat4.mul(
        this.finalMatrix,
        this.getAnimModelMatrix(),
        this.getInvertedModelMatrix(poseId)
      );
    }
    return this.finalMatrix;
  }
  getRotation(poseId) {
    return this.rotations[poseId];
  }
  getTranslation(poseId) {
    return this.translations[poseId];
  }
  getScaling(poseId) {
    return this.scalings[poseId];
  }
};

// ../../../tsps-work/tsps/client/rs/model/skeletal/SkeletalBase.ts
var SkeletalBase = class {
  constructor(buffer, count) {
    this.bones = new Array(count);
    this.poseCount = buffer.readUnsignedByte();
    for (let i = 0; i < this.bones.length; i++) {
      this.bones[i] = new SkeletalBone(this.poseCount, buffer, false);
    }
    this.linkBones();
  }
  linkBones() {
    for (let i = 0; i < this.bones.length; i++) {
      const bone = this.bones[i];
      if (bone.parentId >= 0) {
        bone.parent = this.bones[bone.parentId];
      }
    }
  }
  updateAnimMatrices(skeletalSeq, frame, masks = void 0, mask = false) {
    const poseId = skeletalSeq.poseId;
    let boneIndex = 0;
    for (const bone of this.bones) {
      if (masks === void 0 || masks[boneIndex] === mask) {
        skeletalSeq.updateAnimMatrix(frame, bone, boneIndex, poseId);
      }
      boneIndex++;
    }
  }
  getBoneCount() {
    return this.bones.length;
  }
  getBone(id) {
    if (id >= this.getBoneCount()) {
      return void 0;
    }
    return this.bones[id];
  }
};

// ../../../tsps-work/tsps/client/rs/model/seq/SeqBase.ts
var SeqBase = class {
  constructor(id, count, types, transformActor, masks, labels, skeletalBase) {
    this.id = id;
    this.count = count;
    this.types = types;
    this.transformActor = transformActor;
    this.masks = masks;
    this.labels = labels;
    this.skeletalBase = skeletalBase;
  }
};
var Dat2SeqBase = class {
  static load(cacheInfo, id, data) {
    const buf = new ByteBuffer(data);
    const count = buf.readUnsignedByte();
    const types = new Array(count);
    const transformActor = new Array(count).fill(false);
    const masks = new Uint16Array(count);
    const labels = new Array(count);
    for (let i = 0; i < count; i++) {
      types[i] = buf.readUnsignedByte();
      if (types[i] === 6 /* TYPE_6 */) {
        types[i] = 2 /* ROTATE */;
      }
    }
    if (cacheInfo.game === "runescape" && cacheInfo.revision >= 481) {
      for (let i = 0; i < count; i++) {
        transformActor[i] = buf.readUnsignedByte() === 1;
      }
    } else {
      transformActor.fill(true);
    }
    if (cacheInfo.game === "runescape" && cacheInfo.revision >= 530) {
      for (let i = 0; i < count; i++) {
        masks[i] = buf.readUnsignedShort();
      }
    } else {
      masks.fill(-1);
    }
    for (let i = 0; i < count; i++) {
      labels[i] = new Array(buf.readUnsignedByte());
    }
    for (let i = 0; i < count; i++) {
      for (let l = 0; l < labels[i].length; l++) {
        labels[i][l] = buf.readUnsignedByte();
      }
    }
    let skeletalBase;
    if (buf.remaining > 0) {
      const boneCount = buf.readUnsignedShort();
      if (boneCount > 0) {
        skeletalBase = new SkeletalBase(buf, boneCount);
      }
    }
    return new SeqBase(id, count, types, transformActor, masks, labels, skeletalBase);
  }
};

// ../../../tsps-work/tsps/client/rs/model/seq/SeqFrame.ts
var SeqFrame = class {
  constructor(frameLength, base, transformCount, transformGroups, transformX, transformY, transformZ, resetOriginGroups, hasAlphaTransform, hasColorTransform = false) {
    this.frameLength = frameLength;
    this.base = base;
    this.transformCount = transformCount;
    this.transformGroups = transformGroups;
    this.transformX = transformX;
    this.transformY = transformY;
    this.transformZ = transformZ;
    this.resetOriginGroups = resetOriginGroups;
    this.hasAlphaTransform = hasAlphaTransform;
    this.hasColorTransform = hasColorTransform;
  }
  static {
    this.transformGroupCache = new Int32Array(500);
  }
  static {
    this.transformXCache = new Int32Array(500);
  }
  static {
    this.transformYCache = new Int32Array(500);
  }
  static {
    this.transformZCache = new Int32Array(500);
  }
  static {
    this.resetOriginGroupsCache = new Int32Array(500);
  }
};
var Dat2SeqFrame = class {
  static load(cacheInfo, baseLoader, data) {
    const buf = new ByteBuffer(data);
    const dataBuf = new ByteBuffer(data);
    if (cacheInfo.game === "runescape" && cacheInfo.revision >= 610) {
      buf.readUnsignedByte();
    }
    const baseId = buf.readUnsignedShort();
    const base = baseLoader.load(baseId);
    if (!base) {
      throw new Error("Invalid frame base id: " + baseId);
    }
    const count = buf.readUnsignedByte();
    dataBuf.offset = buf.offset + count;
    let transformCount = 0;
    let resetOriginGroup = -1;
    let lastResetOriginGroup = -1;
    let hasAlphaTransform = false;
    let hasColorTransform = false;
    for (let i = 0; i < count; i++) {
      const type = base.types[i];
      if (type === 0 /* ORIGIN */) {
        resetOriginGroup = i;
      }
      const flag = buf.readUnsignedByte();
      if (flag === 0) {
        continue;
      }
      if (type === 0 /* ORIGIN */) {
        lastResetOriginGroup = i;
      }
      SeqFrame.transformGroupCache[transformCount] = i;
      let defaultValue = 0;
      if (type === 3 /* SCALE */ || type === 10 /* TYPE_10 */) {
        defaultValue = 128;
      }
      if ((flag & 1) !== 0) {
        SeqFrame.transformXCache[transformCount] = dataBuf.readSmart2();
      } else {
        SeqFrame.transformXCache[transformCount] = defaultValue;
      }
      if ((flag & 2) !== 0) {
        SeqFrame.transformYCache[transformCount] = dataBuf.readSmart2();
      } else {
        SeqFrame.transformYCache[transformCount] = defaultValue;
      }
      if ((flag & 4) !== 0) {
        SeqFrame.transformZCache[transformCount] = dataBuf.readSmart2();
      } else {
        SeqFrame.transformZCache[transformCount] = defaultValue;
      }
      if (cacheInfo.game === "runescape" && cacheInfo.revision >= 610) {
        if (type === 0 /* ORIGIN */ || type === 1 /* TRANSLATE */) {
          SeqFrame.transformXCache[transformCount] >>= 2;
          SeqFrame.transformYCache[transformCount] >>= 2;
          SeqFrame.transformZCache[transformCount] >>= 2;
        } else if (type === 2 /* ROTATE */) {
          SeqFrame.transformXCache[transformCount] >>= 4;
          SeqFrame.transformYCache[transformCount] >>= 4;
          SeqFrame.transformZCache[transformCount] >>= 4;
        } else if (type === 3 /* SCALE */) {
        }
      }
      SeqFrame.resetOriginGroupsCache[transformCount] = -1;
      if (type === 1 /* TRANSLATE */ || type === 2 /* ROTATE */ || type === 3 /* SCALE */) {
        if (resetOriginGroup > lastResetOriginGroup) {
          SeqFrame.resetOriginGroupsCache[transformCount] = resetOriginGroup;
          lastResetOriginGroup = resetOriginGroup;
        }
      } else if (type === 5 /* ALPHA */) {
        hasAlphaTransform = true;
      } else if (type === 7 /* LIGHT */) {
        hasColorTransform = true;
      }
      transformCount++;
    }
    if (count !== 0 && dataBuf.offset !== data.length) {
      throw new Error(
        "SeqFrame: Mismatched buffer offset: " + data.length + ", " + dataBuf.offset + ", " + count + ", " + baseId
      );
    }
    const transformGroups = new Array(transformCount);
    const transformX = new Array(transformCount);
    const transformY = new Array(transformCount);
    const transformZ = new Array(transformCount);
    const resetOriginGroups = new Array(transformCount);
    for (let i = 0; i < transformCount; i++) {
      transformGroups[i] = SeqFrame.transformGroupCache[i];
      transformX[i] = SeqFrame.transformXCache[i];
      transformY[i] = SeqFrame.transformYCache[i];
      transformZ[i] = SeqFrame.transformZCache[i];
      resetOriginGroups[i] = SeqFrame.resetOriginGroupsCache[i];
    }
    return new SeqFrame(
      0,
      base,
      transformCount,
      transformGroups,
      transformX,
      transformY,
      transformZ,
      resetOriginGroups,
      hasAlphaTransform,
      hasColorTransform
    );
  }
};

// pose.ts
var input = JSON.parse(import_node_fs.default.readFileSync(0, "utf8"));
var bytes = (s) => Int8Array.from(Buffer.from(s, "base64"));
var info = { game: "runescape", revision: input.rev };
var bases = { load: (id) => Dat2SeqBase.load(info, id, bytes(input.bases[id])) };
var bind = ModelData.decode(bytes(input.model)).light({ isSd: () => true }, 64, 850, -30, -50, -30);
var result = input.frames.map((data) => {
  const frame = Dat2SeqFrame.load(info, bases, bytes(data));
  const posed = Model.copyAnimated(bind, false, false);
  posed.animate(frame, void 0, input.op14);
  return {
    vertices: Array.from(posed.verticesX, (x, i) => [x, posed.verticesY[i], posed.verticesZ[i]]),
    alpha: Array.from(posed.faceAlphas || [], (x) => x & 255),
    colors: Array.from(posed.faceColors || [], (x) => x & 65535)
  };
});
process.stdout.write(JSON.stringify(result));

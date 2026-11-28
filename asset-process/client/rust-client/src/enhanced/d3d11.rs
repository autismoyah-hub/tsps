//! Windows D3D11 backend, isolated from Classic. Uses Microsoft's typed COM API.
use super::*;
use std::{
    hash::{Hash, Hasher},
    mem::{size_of, zeroed},
    ptr,
};
use windows::{
    core::{s, Interface, PCSTR},
    Win32::{
        Foundation::HMODULE,
        Graphics::{Direct3D::Fxc::*, Direct3D::*, Direct3D11::*, Dxgi::Common::*, Dxgi::*},
    },
};
type R<T> = Result<T, String>;
// Every reconstructed palette channel is an 8-bit value. Keep the exact prior
// f32 transfer function, evaluated once per channel value rather than per vertex.
fn linear_channels()->&'static [f32;256] {
    static TABLE:std::sync::OnceLock<[f32;256]>=std::sync::OnceLock::new();
    TABLE.get_or_init(||std::array::from_fn(|i| {
        let x=i as f32/255.;
        if x<=0.04045 {x/12.92} else {((x+0.055)/1.055).powf(2.4)}
    }))
}
fn linear_rgb(color:u32, channels:&[f32;256])->[f32;3] {
    [channels[((color>>16)&255)as usize],channels[((color>>8)&255)as usize],channels[(color&255)as usize]]
}
// Accumulation order is unchanged; finalize each shared vertex once after all
// face contributions, before any vertex consumes it. No additional allocation.
fn finalize_vertex_normals(normals: &mut HashMap<(u64, u16, [u32; 3]), [f32; 3]>) {
    for sum in normals.values_mut() {
        *sum = normal(*sum);
    }
}
fn err(e: windows::core::Error) -> String {
    format!("D3D11 {e}")
}
unsafe fn blob(b: &ID3DBlob) -> &[u8] {
    std::slice::from_raw_parts(b.GetBufferPointer().cast(), b.GetBufferSize())
}
// Preserve S_FALSE: windows0.58's Result wrapper maps it to Ok(()).
// Caller supplies the data type declared by this query; no polling/flush loop.
unsafe fn query_status<T>(context:&ID3D11DeviceContext, query:&ID3D11Query, data:&mut T)->i32 {
    (context.vtable().GetData)(context.as_raw(),query.as_raw(),(data as *mut T).cast(),size_of::<T>()as u32,D3D11_ASYNC_GETDATA_DONOTFLUSH.0 as u32).0
}
fn shader(entry: &'static [u8], target: &'static [u8]) -> R<ID3DBlob> {
    unsafe {
        let source = include_bytes!("shaders.hlsl");
        let mut code = None;
        let mut errors = None;
        let result = D3DCompile(
            source.as_ptr().cast(),
            source.len(),
            s!("tower-enhanced"),
            None,
            None,
            PCSTR(entry.as_ptr()),
            PCSTR(target.as_ptr()),
            D3DCOMPILE_OPTIMIZATION_LEVEL3,
            0,
            &mut code,
            Some(&mut errors),
        );
        if let Err(e) = result {
            return Err(errors
                .as_ref()
                .map(|b| String::from_utf8_lossy(blob(b)).into_owned())
                .unwrap_or_else(|| err(e)));
        }
        code.ok_or("Shader compiler returned no code".into())
    }
}
pub fn adapters() -> R<Vec<Adapter>> {
    unsafe {
        let f: IDXGIFactory1 = CreateDXGIFactory1().map_err(err)?;
        let mut rows = Vec::new();
        for index in 0..16 {
            let Ok(a) = f.EnumAdapters1(index) else { break };
            let d = a.GetDesc1().map_err(err)?;
            let name = String::from_utf16_lossy(&d.Description)
                .trim_end_matches('\0')
                .to_string();
            let mut dev = None;
            let mut level = D3D_FEATURE_LEVEL_10_0;
            let supported = D3D11CreateDevice(
                &a,
                D3D_DRIVER_TYPE_UNKNOWN,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&[
                    D3D_FEATURE_LEVEL_11_1,
                    D3D_FEATURE_LEVEL_11_0,
                    D3D_FEATURE_LEVEL_10_1,
                    D3D_FEATURE_LEVEL_10_0,
                ]),
                D3D11_SDK_VERSION,
                Some(&mut dev),
                Some(&mut level),
                None,
            )
            .is_ok();
            rows.push(Adapter { index, name, vendor_id:d.VendorId, device_id:d.DeviceId, dedicated_bytes:d.DedicatedVideoMemory, shared_bytes:d.SharedSystemMemory, software:d.Flags&2!=0, d3d11:supported, feature_level:level.0 });
        }
        Ok(rows)
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    p: [f32; 3],
    n: [f32; 3],
    color: [f32; 3],
    uv: [f32; 2],
    tex: f32,
    alpha: f32,
    id: u32,
}
struct Target {
    texture: ID3D11Texture2D,
    rtv: Option<ID3D11RenderTargetView>,
    srv: Option<ID3D11ShaderResourceView>,
}
struct Targets {
    size: [u32; 2],
    internal: [u32; 2],
    color: Target,
    normal: Target,
    ids: Target,
    depth: ID3D11DepthStencilView,
    output: Target,
    background: Option<Target>,
    staging: ID3D11Texture2D,
    id_staging: ID3D11Texture2D,
    shadow: Target,
    shadow_dsv: ID3D11DepthStencilView,
    shadow_size: u32,
}
#[derive(Default)]
struct Geometry {
    buffer: Option<ID3D11Buffer>,
    count: u32,
    opaque: u32,
    hash: u64,
    revision: Option<u64>,
    tags: Vec<(u64, u16)>,
    bytes: u64,
}
fn prefix_history_budget(mode:Mode)->u64 {if mode==Mode::Lite {8<<20}else{32<<20}}
fn retain_prefix(history:&mut Option<Geometry>,previous:Geometry,mode:Mode) {
    if previous.buffer.is_some() && previous.count>0 && previous.count==previous.opaque
        && previous.bytes<=prefix_history_budget(mode) {*history=Some(previous);}
}
pub struct Gpu {
    device: ID3D11Device,
    unlit_palette: Vec<u32>,
    context: ID3D11DeviceContext,
    pub identity: String,
    vs: ID3D11VertexShader,
    ps: ID3D11PixelShader,
    shadow_vs: ID3D11VertexShader,
    shadow_ps: ID3D11PixelShader,
    post_vs: ID3D11VertexShader,
    post_ps: ID3D11PixelShader,
    layout: ID3D11InputLayout,
    constants: ID3D11Buffer,
    raster: ID3D11RasterizerState,
    linear: ID3D11SamplerState,
    point: ID3D11SamplerState,
    depth_on: ID3D11DepthStencilState,
    depth_off: ID3D11DepthStencilState,
    blend: ID3D11BlendState,
    targets: Option<Targets>,
    meshes: [Geometry; 4],
    prefix_history: Option<Geometry>,
    shadow_casters: Geometry,
    texture: Option<ID3D11ShaderResourceView>,
    texture_ids: HashMap<u16, u32>,
    texture_size: u32,
    texture_revisions: HashMap<u16, u64>,
    atlas: Option<ID3D11Texture2D>,
    tags: Vec<(u64, u16)>,
    pub last_ids: Vec<u32>,
    pub id_size: [u32; 2],
    query_start: ID3D11Query,
    query_end: ID3D11Query,
    query_disjoint: ID3D11Query,
}
impl Gpu {
    pub fn new(settings: &Settings) -> R<Self> {
        unsafe {
            settings.validate()?;
            let factory: IDXGIFactory1 = CreateDXGIFactory1().map_err(err)?;
            let adapter = if let Some(i) = settings.adapter {
                factory.EnumAdapters1(i).map_err(err)?
            } else {
                let mut selected = None;
                for i in 0..16 {
                    let Ok(a) = factory.EnumAdapters1(i) else {
                        break;
                    };
                    let d = a.GetDesc1().map_err(err)?;
                    if d.Flags & 2 == 0
                        && selected
                            .as_ref()
                            .is_none_or(|(_, bytes)| d.DedicatedVideoMemory > *bytes)
                    {
                        selected = Some((a, d.DedicatedVideoMemory));
                    }
                }
                selected.ok_or("No hardware adapter")?.0
            };
            let desc = adapter.GetDesc1().map_err(err)?;
            if desc.Flags & 2 != 0 {
                return Err("Software adapters must not masquerade as hardware".into());
            }
            let identity = String::from_utf16_lossy(&desc.Description)
                .trim_end_matches('\0')
                .to_string();
            let (mut device, mut context) = (None, None);
            let mut level = D3D_FEATURE_LEVEL_10_0;
            D3D11CreateDevice(
                &adapter,
                D3D_DRIVER_TYPE_UNKNOWN,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&[
                    D3D_FEATURE_LEVEL_11_1,
                    D3D_FEATURE_LEVEL_11_0,
                    D3D_FEATURE_LEVEL_10_1,
                    D3D_FEATURE_LEVEL_10_0,
                ]),
                D3D11_SDK_VERSION,
                Some(&mut device),
                Some(&mut level),
                Some(&mut context),
            )
            .map_err(err)?;
            let device = device.unwrap();
            let context = context.unwrap();
            let mut vs = None;
            let mut ps = None;
            let mut shadow_vs = None;
            let mut shadow_ps = None;
            let mut post_vs = None;
            let mut post_ps = None;
            let vertex = shader(b"worldVS\0", b"vs_4_0\0")?;
            device
                .CreateVertexShader(blob(&vertex), None, Some(&mut vs))
                .map_err(err)?;
            device
                .CreatePixelShader(
                    blob(&shader(b"worldPS\0", b"ps_4_0\0")?),
                    None,
                    Some(&mut ps),
                )
                .map_err(err)?;
            device
                .CreateVertexShader(
                    blob(&shader(b"shadowVS\0", b"vs_4_0\0")?),
                    None,
                    Some(&mut shadow_vs),
                )
                .map_err(err)?;
            device
                .CreatePixelShader(
                    blob(&shader(b"shadowPS\0", b"ps_4_0\0")?),
                    None,
                    Some(&mut shadow_ps),
                )
                .map_err(err)?;
            device
                .CreateVertexShader(
                    blob(&shader(b"postVS\0", b"vs_4_0\0")?),
                    None,
                    Some(&mut post_vs),
                )
                .map_err(err)?;
            device
                .CreatePixelShader(
                    blob(&shader(b"postPS\0", b"ps_4_0\0")?),
                    None,
                    Some(&mut post_ps),
                )
                .map_err(err)?;
            let names = [
                s!("POSITION"),
                s!("NORMAL"),
                s!("COLOR"),
                s!("TEXCOORD"),
                s!("TEXINDEX"),
                s!("ALPHA"),
                s!("OBJECTID"),
            ];
            let formats = [
                DXGI_FORMAT_R32G32B32_FLOAT,
                DXGI_FORMAT_R32G32B32_FLOAT,
                DXGI_FORMAT_R32G32B32_FLOAT,
                DXGI_FORMAT_R32G32_FLOAT,
                DXGI_FORMAT_R32_FLOAT,
                DXGI_FORMAT_R32_FLOAT,
                DXGI_FORMAT_R32_UINT,
            ];
            let offsets = [0, 12, 24, 36, 44, 48, 52];
            let inputs: Vec<_> = (0..7)
                .map(|i| D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: names[i],
                    SemanticIndex: 0,
                    Format: formats[i],
                    InputSlot: 0,
                    AlignedByteOffset: offsets[i],
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                })
                .collect();
            let mut layout = None;
            device
                .CreateInputLayout(&inputs, blob(&vertex), Some(&mut layout))
                .map_err(err)?;
            let mut constants = None;
            device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: 176,
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
                        ..zeroed()
                    },
                    None,
                    Some(&mut constants),
                )
                .map_err(err)?;
            let mut raster = None;
            device
                .CreateRasterizerState(
                    &D3D11_RASTERIZER_DESC {
                        FillMode: D3D11_FILL_SOLID,
                        CullMode: D3D11_CULL_BACK,
                        FrontCounterClockwise: true.into(),
                        DepthClipEnable: true.into(),
                        ..zeroed()
                    },
                    Some(&mut raster),
                )
                .map_err(err)?;
            let mut linear = None;
            let mut point = None;
            let sd = D3D11_SAMPLER_DESC {
                Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
                AddressU: D3D11_TEXTURE_ADDRESS_WRAP,
                AddressV: D3D11_TEXTURE_ADDRESS_WRAP,
                AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
                MaxLOD: f32::MAX,
                MaxAnisotropy: 1,
                ComparisonFunc: D3D11_COMPARISON_NEVER,
                ..zeroed()
            };
            device
                .CreateSamplerState(&sd, Some(&mut linear))
                .map_err(err)?;
            device
                .CreateSamplerState(
                    &D3D11_SAMPLER_DESC {
                        Filter: D3D11_FILTER_MIN_MAG_MIP_POINT,
                        AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
                        AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
                        ..sd
                    },
                    Some(&mut point),
                )
                .map_err(err)?;
            let ds = D3D11_DEPTH_STENCIL_DESC {
                DepthEnable: true.into(),
                DepthWriteMask: D3D11_DEPTH_WRITE_MASK_ALL,
                DepthFunc: D3D11_COMPARISON_LESS_EQUAL,
                ..zeroed()
            };
            let (mut depth_on, mut depth_off) = (None, None);
            device
                .CreateDepthStencilState(&ds, Some(&mut depth_on))
                .map_err(err)?;
            device
                .CreateDepthStencilState(
                    &D3D11_DEPTH_STENCIL_DESC {
                        DepthEnable: false.into(),
                        ..ds
                    },
                    Some(&mut depth_off),
                )
                .map_err(err)?;
            let mut blend_desc: D3D11_BLEND_DESC = zeroed();
            blend_desc.IndependentBlendEnable = true.into();
            for rt in &mut blend_desc.RenderTarget {
                rt.RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8;
            }
            blend_desc.RenderTarget[0].BlendEnable = true.into();
            blend_desc.RenderTarget[0].SrcBlend = D3D11_BLEND_SRC_ALPHA;
            blend_desc.RenderTarget[0].DestBlend = D3D11_BLEND_INV_SRC_ALPHA;
            blend_desc.RenderTarget[0].BlendOp = D3D11_BLEND_OP_ADD;
            blend_desc.RenderTarget[0].SrcBlendAlpha = D3D11_BLEND_ONE;
            blend_desc.RenderTarget[0].DestBlendAlpha = D3D11_BLEND_INV_SRC_ALPHA;
            blend_desc.RenderTarget[0].BlendOpAlpha = D3D11_BLEND_OP_ADD;
            let mut blend = None;
            device
                .CreateBlendState(&blend_desc, Some(&mut blend))
                .map_err(err)?;
            let query = |kind| -> R<ID3D11Query> {
                let mut q = None;
                device
                    .CreateQuery(
                        &D3D11_QUERY_DESC {
                            Query: kind,
                            MiscFlags: 0,
                        },
                        Some(&mut q),
                    )
                    .map_err(err)?;
                Ok(q.unwrap())
            };
            let query_start = query(D3D11_QUERY_TIMESTAMP)?;
            let query_end = query(D3D11_QUERY_TIMESTAMP)?;
            let query_disjoint = query(D3D11_QUERY_TIMESTAMP_DISJOINT)?;
            Ok(Self {
                unlit_palette: crate::floor::palette(1.).map_err(|e| e.to_string())?,
                device,
                context,
                identity,
                vs: vs.unwrap(),
                ps: ps.unwrap(),
                shadow_vs: shadow_vs.unwrap(),
                shadow_ps: shadow_ps.unwrap(),
                post_vs: post_vs.unwrap(),
                post_ps: post_ps.unwrap(),
                layout: layout.unwrap(),
                constants: constants.unwrap(),
                raster: raster.unwrap(),
                linear: linear.unwrap(),
                point: point.unwrap(),
                depth_on: depth_on.unwrap(),
                depth_off: depth_off.unwrap(),
                blend: blend.unwrap(),
                targets: None,
                meshes: [Geometry::default(), Geometry::default(), Geometry::default(), Geometry::default()],
                prefix_history: None,
                shadow_casters: Geometry::default(),

                texture: None,
                texture_ids: HashMap::new(),
                texture_size: 0,
                texture_revisions: HashMap::new(),
                atlas: None,
                tags: Vec::new(),
                last_ids: Vec::new(),
                id_size: [0; 2],
                query_start,
                query_end,
                query_disjoint,
            })
        }
    }
    fn target(&self, size: [u32; 2], format: DXGI_FORMAT, bind: u32) -> R<Target> {
        unsafe {
            let mut texture = None;
            self.device
                .CreateTexture2D(
                    &D3D11_TEXTURE2D_DESC {
                        Width: size[0],
                        Height: size[1],
                        MipLevels: 1,
                        ArraySize: 1,
                        Format: format,
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: bind,
                        ..zeroed()
                    },
                    None,
                    Some(&mut texture),
                )
                .map_err(err)?;
            let texture = texture.unwrap();
            let mut rtv = None;
            let mut srv = None;
            if bind & D3D11_BIND_RENDER_TARGET.0 as u32 != 0 {
                self.device
                    .CreateRenderTargetView(&texture, None, Some(&mut rtv))
                    .map_err(err)?;
            }
            if bind & D3D11_BIND_SHADER_RESOURCE.0 as u32 != 0 {
                self.device
                    .CreateShaderResourceView(&texture, None, Some(&mut srv))
                    .map_err(err)?;
            }
            Ok(Target { texture, rtv, srv })
        }
    }
    fn staging(&self, texture: &ID3D11Texture2D) -> R<ID3D11Texture2D> {
        unsafe {
            let mut d = zeroed();
            texture.GetDesc(&mut d);
            d.Usage = D3D11_USAGE_STAGING;
            d.BindFlags = 0;
            d.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
            let mut out = None;
            self.device
                .CreateTexture2D(&d, None, Some(&mut out))
                .map_err(err)?;
            Ok(out.unwrap())
        }
    }
    fn resize(&mut self, size: [u32; 2], settings: &Settings) -> R<()> {
        unsafe {
            let internal = size.map(|n| ((n as f32 * settings.scale).round() as u32).max(1));
            if size.contains(&0) || internal[0] as u64 * internal[1] as u64 > 8_294_400 {
                return Err("Enhanced render target exceeds bounded 8.3M pixel allocation".into());
            }
            if self.targets.as_ref().is_some_and(|t| {
                t.size == size && t.internal == internal && t.shadow_size == settings.shadows.max(1)
            }) {
                return Ok(());
            }
            self.context.OMSetRenderTargets(None, None);
            self.context
                .PSSetShaderResources(0, Some(&[None, None, None, None, None]));
            self.targets = None;
            let bind = (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32;
            let color = self.target(internal, DXGI_FORMAT_R16G16B16A16_FLOAT, bind)?;
            let normal = self.target(internal, DXGI_FORMAT_R16G16B16A16_FLOAT, bind)?;
            let ids = self.target(
                internal,
                DXGI_FORMAT_R32_UINT,
                D3D11_BIND_RENDER_TARGET.0 as u32,
            )?;
            let output = self.target(
                size,
                DXGI_FORMAT_B8G8R8A8_UNORM,
                D3D11_BIND_RENDER_TARGET.0 as u32,
            )?;
            let depth = self.target(
                internal,
                DXGI_FORMAT_D32_FLOAT,
                D3D11_BIND_DEPTH_STENCIL.0 as u32,
            )?;
            let mut dsv = None;
            self.device
                .CreateDepthStencilView(&depth.texture, None, Some(&mut dsv))
                .map_err(err)?;
            let shadow_size = settings.shadows.max(1);
            let mut shadow_texture = None;
            self.device
                .CreateTexture2D(
                    &D3D11_TEXTURE2D_DESC {
                        Width: shadow_size,
                        Height: shadow_size,
                        MipLevels: 1,
                        ArraySize: 1,
                        Format: DXGI_FORMAT_R32_TYPELESS,
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: (D3D11_BIND_DEPTH_STENCIL.0 | D3D11_BIND_SHADER_RESOURCE.0)
                            as u32,
                        ..zeroed()
                    },
                    None,
                    Some(&mut shadow_texture),
                )
                .map_err(err)?;
            let texture = shadow_texture.unwrap();
            let mut shadow_dsv = None;
            self.device
                .CreateDepthStencilView(
                    &texture,
                    Some(&D3D11_DEPTH_STENCIL_VIEW_DESC {
                        Format: DXGI_FORMAT_D32_FLOAT,
                        ViewDimension: D3D11_DSV_DIMENSION_TEXTURE2D,
                        ..zeroed()
                    }),
                    Some(&mut shadow_dsv),
                )
                .map_err(err)?;
            let mut shadow_srv = None;
            let mut sv: D3D11_SHADER_RESOURCE_VIEW_DESC = zeroed();
            sv.Format = DXGI_FORMAT_R32_FLOAT;
            sv.ViewDimension = D3D_SRV_DIMENSION_TEXTURE2D;
            sv.Anonymous.Texture2D = D3D11_TEX2D_SRV {
                MostDetailedMip: 0,
                MipLevels: 1,
            };
            self.device
                .CreateShaderResourceView(&texture, Some(&sv), Some(&mut shadow_srv))
                .map_err(err)?;
            let shadow = Target {
                texture,
                rtv: None,
                srv: shadow_srv,
            };
            let staging = self.staging(&output.texture)?;
            let id_staging = self.staging(&ids.texture)?;
            self.targets = Some(Targets {
                size,
                internal,
                color,
                normal,
                ids,
                depth: dsv.unwrap(),
                output,
                background: None,
                staging,
                id_staging,
                shadow,
                shadow_dsv: shadow_dsv.unwrap(),
                shadow_size,
            });
            Ok(())
        }
    }
    fn textures(
        &mut self,
        scene: &SceneFrame<'_>,
        settings: &Settings,
        stats: &mut Stats,
    ) -> R<()> {
        unsafe {
            let same = self.texture.is_some()
                && self.texture_ids.len() == scene.textures.len()
                && self.texture_size == settings.texture_size
                && scene
                    .textures
                    .keys()
                    .all(|id| self.texture_ids.contains_key(id));
            if same {
                let n = settings.texture_size;
                for (&id, t) in scene.textures {
                    if self.texture_revisions.get(&id) == Some(&texture_revision(t)) {
                        continue;
                    }
                    let mut data = vec![0u32; (n * n) as usize];
                    for y in 0..n {
                        for x in 0..n {
                            let p = t.pixels[(y as usize * t.size / n as usize) * t.size
                                + x as usize * t.size / n as usize];
                            data[(y * n + x) as usize] = texture_rgba(p, t.definition.opaque);
                        }
                    }
                    self.context.UpdateSubresource(
                        self.atlas.as_ref().unwrap(),
                        self.texture_ids[&id],
                        None,
                        data.as_ptr().cast(),
                        n * 4,
                        n * n * 4,
                    );
                    self.texture_revisions.insert(id, texture_revision(t));
                }
                return Ok(());
            }
            self.meshes = [Geometry::default(), Geometry::default(), Geometry::default(), Geometry::default()];
            self.prefix_history=None;
            self.shadow_casters=Geometry::default();
            self.texture_revisions = scene
                .textures
                .iter()
                .map(|(&id, t)| (id, texture_revision(t)))
                .collect();
            if scene.textures.len() > 511 {
                return Err("Texture array bound of 511 exceeded".into());
            }
            self.texture_ids.clear();
            let n = settings.texture_size;
            let layers = (scene.textures.len() + 1).max(2) as u32;
            let mut all = vec![0xffffffffu32; (n * n * layers) as usize];
            let mut keys: Vec<_> = scene.textures.keys().copied().collect();
            keys.sort_unstable();
            for (index, id) in keys.into_iter().enumerate() {
                self.texture_ids.insert(id, index as u32);
                let texture = &scene.textures[&id];
                for y in 0..n {
                    for x in 0..n {
                        let p = texture.pixels[(y as usize * texture.size / n as usize)
                            * texture.size
                            + x as usize * texture.size / n as usize];
                        let rgba = texture_rgba(p, texture.definition.opaque);
                        all[(index * n as usize * n as usize) + (y * n + x) as usize] = rgba;
                    }
                }
            }
            let data: Vec<_> = (0..layers)
                .map(|i| D3D11_SUBRESOURCE_DATA {
                    pSysMem: all.as_ptr().add((i * n * n) as usize).cast(),
                    SysMemPitch: n * 4,
                    SysMemSlicePitch: n * n * 4,
                })
                .collect();
            let mut texture = None;
            self.device
                .CreateTexture2D(
                    &D3D11_TEXTURE2D_DESC {
                        Width: n,
                        Height: n,
                        MipLevels: 1,
                        ArraySize: layers,
                        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
                        ..zeroed()
                    },
                    Some(data.as_ptr()),
                    Some(&mut texture),
                )
                .map_err(err)?;
            let mut srv = None;
            self.device
                .CreateShaderResourceView(texture.as_ref().unwrap(), None, Some(&mut srv))
                .map_err(err)?;
            self.texture = srv;
            self.atlas = texture;
            self.texture_size = n;
            stats.texture_bytes = (n * n * layers * 4) as u64;
            Ok(())
        }
    }
    fn geometry(
        device: &ID3D11Device,
        unlit_palette: &[u32],
        texture_ids: &HashMap<u16, u32>,
        scene: &SceneFrame<'_>,
        settings: &Settings,
        stats: &mut Stats,
        mesh: &mut Geometry,
        revision: Option<u64>,
        mut history: Option<&mut Option<Geometry>>,
    ) -> R<()> {
        unsafe {
            let camera_key = if scene.triangles.iter().any(|t| t.alpha > 0 && t.alpha < 255) {
                Some((
                    scene.camera.position.map(f32::to_bits),
                    scene.camera.transform([0., 0., 0.]).map(f32::to_bits),
                    scene.camera.transform([0., 0., 1.]).map(f32::to_bits),
                ))
            } else {
                None
            };
            // Fallback HSL reconstruction bakes ambient lighting into vertices.
            // Explicit materials and textured faces do not use this input.
            let ambient_key = (scene.albedo.is_none() && scene.triangles.iter().any(|t|
                t.texture.is_none() && t.material_hsl.is_none()
                    && t.alpha != 255 && t.colors[0] != 12345678)).then_some(scene.ambient);
            let revision = revision.map(|r| {
                let mut h = std::hash::DefaultHasher::new();
                r.hash(&mut h);
                camera_key.hash(&mut h);
                ambient_key.hash(&mut h);
                h.finish()
            });
            if mesh.buffer.is_some() && revision.is_some() && revision == mesh.revision {
                return Ok(());
            }
            let mut hash = std::hash::DefaultHasher::new();
            for (i, t) in scene.triangles.iter().enumerate() {
                t.vertices.map(|v| v.map(f32::to_bits)).hash(&mut hash);
                t.colors.hash(&mut hash);
                t.alpha.hash(&mut hash);
                t.flat.hash(&mut hash);
                t.texture.hash(&mut hash);
                t.uv.map(|v| v.map(f32::to_bits)).hash(&mut hash);
                t.tag.hash(&mut hash);
                t.view.hash(&mut hash);
                t.material_hsl.hash(&mut hash);
                scene.albedo.map(|a| a[i]).hash(&mut hash);
            }
            camera_key.hash(&mut hash);
            ambient_key.hash(&mut hash);
            let key = hash.finish();
            if mesh.buffer.is_some() && key == mesh.hash {
                mesh.revision = revision;
                return Ok(());
            }
            if let Some(history)=history.as_deref_mut() {
                if history.as_ref().is_some_and(|cached|cached.hash==key) {
                    let mut restored=history.take().unwrap();restored.revision=revision;
                    let previous=std::mem::replace(mesh,restored);
                    retain_prefix(history,previous,settings.mode);
                    stats.prefix_cache_hits+=1;
                    return Ok(());
                }
            }
            let limit = if settings.mode == Mode::Lite {
                64 << 20
            } else {
                192 << 20
            };
            if scene.triangles.len() * 3 * size_of::<Vertex>() > limit {
                return Err("Geometry exceeds configured bounded vertex budget".into());
            }
            // Keep the previous buffer and its identities intact until the new
            // allocation succeeds; the retained variant is never drawn alongside it.
            let destination=mesh;
            let mut replacement=Geometry::default();
            let mesh=&mut replacement;
            let mut normals: HashMap<(u64, u16, [u32; 3]), [f32; 3]> = HashMap::new();
            let mut face_normals = Vec::with_capacity(scene.triangles.len());
            for t in scene.triangles {
                let n = normal(cross(
                    sub(t.vertices[1], t.vertices[0]),
                    sub(t.vertices[2], t.vertices[0]),
                ));
                face_normals.push(n);
                if !t.flat {
                    for p in t.vertices {
                        let sum = normals
                            .entry((t.tag, t.view, p.map(f32::to_bits)))
                            .or_insert([0.; 3]);
                        for j in 0..3 {
                            sum[j] += n[j];
                        }
                    }
                }
            }
            finalize_vertex_normals(&mut normals);
            let mut tags = HashMap::new();
            mesh.tags = vec![(0, 0)];
            let mut vertices = Vec::with_capacity(scene.triangles.len() * 3);
            let mut transparent = Vec::new();
            let native_light = normal([-30., -50., -30.]);
            let channels=linear_channels();
            for (i, t) in scene.triangles.iter().enumerate() {
                if t.alpha == 255 || t.colors[0] == 12345678 {
                    continue;
                }
                let next = mesh.tags.len() as u32;
                let id = *tags.entry((t.tag, t.view)).or_insert_with(|| {
                    mesh.tags.push((t.tag, t.view));
                    next
                });
                let dst = if t.alpha == 0 {
                    &mut vertices
                } else {
                    &mut transparent
                };
                for j in 0..3 {
                    let n = if t.flat {
                        face_normals[i]
                    } else {
                        normals[&(t.tag, t.view, t.vertices[j].map(f32::to_bits))]
                    };
                    let code = if let Some(base) = scene.albedo {
                        base[i][j]
                    } else if let Some(base)=t.material_hsl {
                        base
                    } else if t.texture.is_none() {
                        let c = t.colors[j].clamp(0, 65535) as u16;
                        let brightness =
                            (scene.ambient as f32 + dot(n, native_light) * 96.).max(16.);
                        let l = ((c & 127) as f32 * 128. / brightness).clamp(2., 126.) as u16;
                        (c & 65408) | l
                    } else {
                        0
                    };
                    let rgb = if t.texture.is_some() {
                        [1.; 3]
                    } else {
                        linear_rgb(unlit_palette[code as usize],channels)
                    };
                    dst.push(Vertex {
                        p: t.vertices[j],
                        n,
                        color: rgb,
                        uv: t.uv[j],
                        tex: t
                            .texture
                            .and_then(|id| texture_ids.get(&id).copied())
                            .map_or(-1., |v| v as f32),
                        alpha: 1. - t.alpha as f32 / 255.,
                        id,
                    });
                }
            }
            mesh.opaque = vertices.len() as u32;
            let mut faces: Vec<_> = transparent.chunks_exact(3).collect();
            faces.sort_by(|a, b| {
                let depth = |v: &[Vertex]| {
                    v.iter()
                        .map(|v| scene.camera.transform(v.p)[2])
                        .sum::<f32>()
                };
                depth(b).total_cmp(&depth(a))
            });
            for face in faces {
                vertices.extend_from_slice(face);
            }
            mesh.count = vertices.len() as u32;
            if vertices.is_empty() {
                return Err("Empty render-facing scene".into());
            }
            let bytes = vertices.len() * size_of::<Vertex>();
            let mut buffer = None;
            device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: bytes as u32,
                        Usage: D3D11_USAGE_IMMUTABLE,
                        BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
                        ..zeroed()
                    },
                    Some(&D3D11_SUBRESOURCE_DATA {
                        pSysMem: vertices.as_ptr().cast(),
                        SysMemPitch: 0,
                        SysMemSlicePitch: 0,
                    }),
                    Some(&mut buffer),
                )
                .map_err(err)?;
            mesh.buffer = buffer;
            mesh.hash = key;
            mesh.bytes = bytes as u64;
            mesh.revision = revision;
            let previous=std::mem::replace(destination,replacement);
            if let Some(history)=history {retain_prefix(history,previous,settings.mode);}
            stats.uploads += 1;
            Ok(())
        }
    }
    // Shadow shaders consume only position, texture coordinates/index and alpha.
    // Do not reconstruct lighting, smooth normals or object identities for this buffer.
    fn shadow_geometry(device: &ID3D11Device, texture_ids: &HashMap<u16,u32>,
        triangles: &[&Triangle], settings: &Settings, stats: &mut Stats, mesh: &mut Geometry) -> R<()> {
        let mut hash=std::hash::DefaultHasher::new();
        for t in triangles {
            t.vertices.map(|p|p.map(f32::to_bits)).hash(&mut hash);
            t.uv.map(|uv|uv.map(f32::to_bits)).hash(&mut hash);
            t.texture.and_then(|id|texture_ids.get(&id).copied()).hash(&mut hash);
        }
        let key=hash.finish();
        if mesh.buffer.is_some() && mesh.hash==key {return Ok(());}
        let bytes=triangles.len()*3*size_of::<Vertex>();
        let limit=if settings.mode==Mode::Lite {64<<20} else {192<<20};
        if bytes>limit {return Err("Shadow geometry exceeds configured bounded vertex budget".into());}
        let mut vertices=Vec::with_capacity(triangles.len()*3);
        for t in triangles {
            let tex=t.texture.and_then(|id|texture_ids.get(&id).copied()).map_or(-1.,|id|id as f32);
            for j in 0..3 {vertices.push(Vertex{p:t.vertices[j],n:[0.;3],color:[0.;3],uv:t.uv[j],tex,alpha:1.,id:0});}
        }
        unsafe {
            let mut buffer=None;
            device.CreateBuffer(&D3D11_BUFFER_DESC{ByteWidth:bytes as u32,Usage:D3D11_USAGE_IMMUTABLE,
                BindFlags:D3D11_BIND_VERTEX_BUFFER.0 as u32,..zeroed()},
                Some(&D3D11_SUBRESOURCE_DATA{pSysMem:vertices.as_ptr().cast(),SysMemPitch:0,SysMemSlicePitch:0}),
                Some(&mut buffer)).map_err(err)?;
            *mesh=Geometry{buffer,count:vertices.len()as u32,opaque:vertices.len()as u32,hash:key,
                revision:None,tags:Vec::new(),bytes:bytes as u64};
        }
        stats.uploads+=1;Ok(())
    }
    pub fn render(
        &mut self,
        scene: &SceneFrame<'_>,
        settings: &Settings,
        stats: &mut Stats,
        background: Option<&[u32]>,
        casters: &[&crate::render3d::Triangle],
    ) -> R<Vec<u32>> {
        unsafe {
            let prepare = Instant::now();
            self.resize(scene.size, settings)?;
            if let Some(pixels) = background {
                if self.targets.as_ref().unwrap().background.is_none() {
                    let target = self.target(scene.size, DXGI_FORMAT_B8G8R8A8_UNORM, D3D11_BIND_SHADER_RESOURCE.0 as u32)?;
                    self.targets.as_mut().unwrap().background = Some(target);
                }
                self.context.UpdateSubresource(&self.targets.as_ref().unwrap().background.as_ref().unwrap().texture,
                    0, None, pixels.as_ptr().cast(), scene.size[0] * 4, 0);
            } else {
                self.targets.as_mut().unwrap().background = None;
            }
            self.textures(scene, settings, stats)?;
            let stable = scene.stable_count.min(scene.triangles.len());
            self.meshes[2]=Geometry::default();
            self.meshes[3]=Geometry::default();
            for index in 0..2 {
                let (start, end, revision) = if index == 0 {
                    (0, stable, scene.stable_revision)
                } else {
                    (stable, scene.triangles.len(), None)
                };
                if start == end {
                    let previous=std::mem::take(&mut self.meshes[index]);
                    if index==0 {retain_prefix(&mut self.prefix_history,previous,settings.mode);}
                    continue;
                }
                let subset = SceneFrame {
                    triangles: &scene.triangles[start..end],
                    albedo: scene.albedo.map(|a| &a[start..end]),
                    ..*scene
                };
                Self::geometry(
                    &self.device,
                    &self.unlit_palette,
                    &self.texture_ids,
                    &subset,
                    settings,
                    stats,
                    &mut self.meshes[index],
                    revision,
                    if index==0 {Some(&mut self.prefix_history)}else{None},
                )?;
            }
            let (light,right,down)=light_basis(settings.light);
            let opaque=if settings.shadows>0 {casters.iter().copied()
                .filter(|t|t.alpha==0 && t.colors[0]!=12345678
                    && intersects_shadow_volume(t.vertices,scene.light_focus,[right,down,light]))
                .collect::<Vec<_>>()} else {Vec::new()};
            if opaque.is_empty() {self.shadow_casters=Geometry::default();}
            else {
                Self::shadow_geometry(&self.device,&self.texture_ids,&opaque,settings,stats,&mut self.shadow_casters)?;
            }
            self.tags = vec![(0, 0)];
            for mesh in &self.meshes {
                if !mesh.tags.is_empty() {
                    self.tags.extend_from_slice(&mesh.tags[1..]);
                }
            }
            stats.retained_prefix_bytes=self.prefix_history.as_ref().map_or(0,|m|m.bytes);
            stats.buffer_bytes = self.meshes.iter().map(|m| m.bytes).sum::<u64>()+self.shadow_casters.bytes+stats.retained_prefix_bytes;
            stats.prepare_ms = prepare.elapsed().as_secs_f64() * 1000.;
            let begin = Instant::now();
            let targets = self.targets.as_ref().unwrap();
            let c = scene.camera;
            let basis = |axis| {
                let mut p = c.position;
                p[axis] += 1.;
                c.transform(p)
            };
            let cols = [basis(0), basis(1), basis(2)];
            let row = |i| [cols[0][i], cols[1][i], cols[2][i], 0.];
            let (light, right, down) = light_basis(settings.light);
            let shadow_extent = 768.;
            let mut constants = [
                [c.position[0], c.position[1], c.position[2], c.near],
                row(0),
                row(1),
                row(2),
                [
                    c.focal * 2. / scene.size[0] as f32,
                    c.focal * 2. / scene.size[1] as f32,
                    c.far as f32,
                    settings.exposure,
                ],
                [light[0], light[1], light[2], 0.],
                [
                    scene.light_focus[0],
                    scene.light_focus[1],
                    scene.light_focus[2],
                    shadow_extent,
                ],
                [right[0], right[1], right[2], if background.is_some() {1.} else {0.}],
                [down[0], down[1], down[2], 0.],
                [
                    settings.shadows as f32,
                    settings.ao as f32,
                    if settings.aa { 1. } else { 0. },
                    if settings.mode == Mode::Gpu { 2. } else { 1. },
                ],
                [
                    targets.internal[0] as f32,
                    targets.internal[1] as f32,
                    scene.size[0] as f32,
                    scene.size[1] as f32,
                ],
            ];
            constants[1][3] =
                (c.screen_offset[0] + (scene.size[0] / 2) as f32 - scene.size[0] as f32 * 0.5) * 2.
                    / scene.size[0] as f32;
            constants[2][3] =
                (c.screen_offset[1] + (scene.size[1] / 2) as f32 - scene.size[1] as f32 * 0.5) * 2.
                    / scene.size[1] as f32;
            self.context.UpdateSubresource(
                &self.constants,
                0,
                None,
                constants.as_ptr().cast(),
                0,
                0,
            );
            self.context
                .VSSetConstantBuffers(0, Some(&[Some(self.constants.clone())]));
            self.context
                .PSSetConstantBuffers(0, Some(&[Some(self.constants.clone())]));
            self.context.RSSetState(&self.raster);
            self.context.IASetInputLayout(&self.layout);
            self.context
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            self.context.PSSetSamplers(
                0,
                Some(&[Some(self.linear.clone()), Some(self.point.clone())]),
            );
            self.context.Begin(&self.query_disjoint);
            self.context.End(&self.query_start);
            self.context
                .PSSetShaderResources(0, Some(&[None, self.texture.clone(), None, None]));
            if settings.shadows > 0 {
                self.context
                    .OMSetRenderTargets(None, Some(&targets.shadow_dsv));
                self.context
                    .ClearDepthStencilView(&targets.shadow_dsv, D3D11_CLEAR_DEPTH.0, 1., 0);
                self.context.RSSetViewports(Some(&[D3D11_VIEWPORT {
                    Width: targets.shadow_size as f32,
                    Height: targets.shadow_size as f32,
                    MinDepth: 0.,
                    MaxDepth: 1.,
                    ..zeroed()
                }]));
                self.context.OMSetDepthStencilState(&self.depth_on, 0);
                self.context.VSSetShader(&self.shadow_vs, None);
                self.context.PSSetShader(&self.shadow_ps, None);
                self.draw_geometry(&mut constants, true);
            }
            self.context.OMSetRenderTargets(
                Some(&[
                    targets.color.rtv.clone(),
                    targets.normal.rtv.clone(),
                    targets.ids.rtv.clone(),
                ]),
                Some(&targets.depth),
            );
            self.context.ClearRenderTargetView(
                targets.color.rtv.as_ref().unwrap(),
                if background.is_some() { &[0.; 4] } else { &[0.035, 0.047, 0.06, 1.] },
            );
            self.context
                .ClearRenderTargetView(targets.normal.rtv.as_ref().unwrap(), &[0.; 4]);
            self.context
                .ClearRenderTargetView(targets.ids.rtv.as_ref().unwrap(), &[0.; 4]);
            self.context
                .ClearDepthStencilView(&targets.depth, D3D11_CLEAR_DEPTH.0, 1., 0);
            self.context.RSSetViewports(Some(&[D3D11_VIEWPORT {
                Width: targets.internal[0] as f32,
                Height: targets.internal[1] as f32,
                MinDepth: 0.,
                MaxDepth: 1.,
                ..zeroed()
            }]));
            self.context.OMSetDepthStencilState(&self.depth_on, 0);
            self.context.OMSetBlendState(&self.blend, None, u32::MAX);
            self.context.PSSetShaderResources(
                0,
                Some(&[targets.shadow.srv.clone(), self.texture.clone(), None, None]),
            );
            self.context.VSSetShader(&self.vs, None);
            self.context.PSSetShader(&self.ps, None);
            self.draw_geometry(&mut constants, false);
            self.context
                .OMSetRenderTargets(Some(&[targets.output.rtv.clone()]), None);
            self.context.OMSetDepthStencilState(&self.depth_off, 0);
            self.context.OMSetBlendState(None, None, u32::MAX);
            self.context.IASetInputLayout(None);
            self.context.RSSetState(None);
            self.context.PSSetShaderResources(
                2,
                Some(&[targets.color.srv.clone(), targets.normal.srv.clone(), targets.background.as_ref().and_then(|t|t.srv.clone())]),
            );
            self.context.VSSetShader(&self.post_vs, None);
            self.context.PSSetShader(&self.post_ps, None);
            self.context.RSSetViewports(Some(&[D3D11_VIEWPORT {
                Width: scene.size[0] as f32,
                Height: scene.size[1] as f32,
                MinDepth: 0.,
                MaxDepth: 1.,
                ..zeroed()
            }]));
            self.context.Draw(3, 0);
            self.context.End(&self.query_end);
            self.context.End(&self.query_disjoint);
            self.context
                .PSSetShaderResources(0, Some(&[None, None, None, None, None]));
            self.context.OMSetRenderTargets(None, None);
            self.context
                .CopyResource(&targets.staging, &targets.output.texture);
            self.context
                .CopyResource(&targets.id_staging, &targets.ids.texture);
            let read = Instant::now();
            let pixels = self.read(&targets.staging, scene.size)?;
            self.last_ids = self.read(&targets.id_staging, targets.internal)?;
            self.id_size = targets.internal;
            stats.readback_ms = read.elapsed().as_secs_f64() * 1000.;
            let mut disjoint: D3D11_QUERY_DATA_TIMESTAMP_DISJOINT = zeroed();
            let mut ts0 = 0u64;
            let mut ts1 = 0u64;
            let status=[
                query_status(&self.context,&self.query_disjoint,&mut disjoint),
                query_status(&self.context,&self.query_start,&mut ts0),
                query_status(&self.context,&self.query_end,&mut ts1),
            ];
            stats.gpu_ms=timestamp_ms(status,disjoint.Frequency,disjoint.Disjoint.as_bool(),ts0,ts1);
            stats.frames += 1;
            stats.triangles = self.meshes.iter().map(|m| m.count as usize / 3).sum();
            stats.extra_shadow_triangles=self.shadow_casters.opaque as usize/3;
            stats.draw_calls = self.meshes.iter().filter(|m| m.count > 0).count() as u32
                + u32::from(settings.shadows > 0)*(self.meshes.iter().filter(|m|m.opaque>0).count()as u32+u32::from(self.shadow_casters.opaque>0)) + 1;
            stats.render_ms = begin.elapsed().as_secs_f64() * 1000.;
            stats.target_bytes = targets.internal[0] as u64 * targets.internal[1] as u64 * 28
                + scene.size[0] as u64 * scene.size[1] as u64 * 8
                + targets.shadow_size as u64 * targets.shadow_size as u64 * 4
                + if background.is_some() {scene.size[0] as u64 * scene.size[1] as u64 * 4} else {0};
            Ok(pixels)
        }
    }
    unsafe fn draw_geometry(&self, constants: &mut [[f32; 4]; 11], shadow: bool) {
        let mut id_offset = 0;
        for mesh in self.meshes.iter().chain(shadow.then_some(&self.shadow_casters)) {
            let count=if shadow{mesh.opaque}else{mesh.count};
            if count==0{continue;}
            if let Some(buffer) = &mesh.buffer {
                constants[5][3] = id_offset as f32;
                self.context.UpdateSubresource(
                    &self.constants,
                    0,
                    None,
                    constants.as_ptr().cast(),
                    0,
                    0,
                );
                let buffers = [Some(buffer.clone())];
                let stride = size_of::<Vertex>() as u32;
                let offset = 0;
                self.context.IASetVertexBuffers(
                    0,
                    1,
                    Some(buffers.as_ptr()),
                    Some(&stride),
                    Some(&offset),
                );
                self.context
                    .Draw(count, 0);
                id_offset += mesh.tags.len().saturating_sub(1);
            }
        }
    }
    unsafe fn read(&self, texture: &ID3D11Texture2D, size: [u32; 2]) -> R<Vec<u32>> {
        let mut m = zeroed();
        self.context
            .Map(texture, 0, D3D11_MAP_READ, 0, Some(&mut m))
            .map_err(err)?;
        let mut pixels = vec![0u32; (size[0] * size[1]) as usize];
        for y in 0..size[1] as usize {
            ptr::copy_nonoverlapping(
                (m.pData as *const u8)
                    .add(y * m.RowPitch as usize)
                    .cast::<u32>(),
                pixels.as_mut_ptr().add(y * size[0] as usize),
                size[0] as usize,
            );
        }
        self.context.Unmap(texture, 0);
        Ok(pixels)
    }
    pub fn diagnostic_ids(&self) -> DiagnosticIds {
        DiagnosticIds { size: self.id_size, values: self.last_ids.iter()
            .map(|&id| self.tags.get(id as usize).copied().unwrap_or((0,0))).collect() }
    }
}
// Match the Classic zero-texel rule before converting RGB to the GPU byte order.
fn texture_rgba(pixel:u32, opaque:bool)->u32 {
    if pixel==0 && !opaque {0} else {0xff000000 | ((pixel&255)<<16) | (pixel&0xff00) | ((pixel>>16)&255)}
}
fn texture_revision(texture: &Texture) -> u64 {
    let mut hash = std::hash::DefaultHasher::new();
    texture.size.hash(&mut hash);
    texture.definition.opaque.hash(&mut hash);
    texture.pixels.hash(&mut hash);
    hash.finish()
}

#[cfg(test)]
mod texture_tests {
    use super::*;
    #[test]
    fn texel_conversion_obeys_classic_opaque_zero_rule() {
        assert_eq!(texture_rgba(0,false),0);
        assert_eq!(texture_rgba(0,true),0xff000000);
        for opaque in [false,true] {
            assert_eq!(texture_rgba(0x123456,opaque),0xff563412);
            assert_eq!(texture_rgba(0xff000000,opaque),0xff000000);
        }
    }
    #[test]
    fn opacity_changes_invalidate_texture_content_identity() {
        let mut t=Texture{definition:crate::texture::Definition{sprite:0,average_hsl:0,opaque:false,direction:0,speed:0},size:64,pixels:vec![0;64*64]};
        let transparent=texture_revision(&t);t.definition.opaque=true;
        assert_ne!(texture_revision(&t),transparent);
        t.definition.opaque=false;assert_eq!(texture_revision(&t),transparent);
    }
}

#[cfg(test)]
mod linear_color_tests {
    use super::*;
    fn original(color:u32)->[u32;3] {
        [(color>>16&255)as f32/255.,(color>>8&255)as f32/255.,(color&255)as f32/255.]
            .map(|x|if x<=0.04045 {x/12.92}else{((x+0.055)/1.055).powf(2.4)})
            .map(f32::to_bits)
    }
    #[test]
    fn every_channel_and_palette_entry_matches_original_vertex_math_bits() {
        let table=linear_channels();
        assert!(std::ptr::eq(table,linear_channels()));
        assert_eq!(table[255].to_bits(),1f32.to_bits());
        for value in 0..256u32 {
            for color in [value,value<<8,value<<16,value*0x010101,0xff000000|value*0x010101] {
                assert_eq!(linear_rgb(color,table).map(f32::to_bits),original(color));
            }
        }
        for color in crate::floor::palette(1.).unwrap() {
            assert_eq!(linear_rgb(color,table).map(f32::to_bits),original(color));
        }
    }
}

#[cfg(test)]
mod shared_normal_tests {
    use super::*;

    #[test]
    fn shared_finalization_preserves_prior_vertex_bits_and_identity() {
        let mut sums = HashMap::new();
        // Repeated positions belong to different objects/views; signed zero is
        // part of the existing exact vertex identity and must remain distinct.
        for tag in [0, 1, u64::MAX] {
            for view in [0, 1, u16::MAX] {
                for i in 0..128u32 {
                    let position = [i as f32, if i % 2 == 0 { 0.0 } else { -0.0 }, 17.0];
                    let mut sum = [0.0; 3];
                    for face in 0..(i % 13 + 1) {
                        let contribution = normal(cross(
                            [i as f32 - 64.0, face as f32 - 6.0, view as f32],
                            [face as f32, 1.0, 17.0 - i as f32],
                        ));
                        for j in 0..3 { sum[j] += contribution[j]; }
                    }
                    sums.insert((tag, view, position.map(f32::to_bits)), sum);
                }
            }
        }
        for (i, sum) in [[0.0,-0.0,0.0], [1e-30,-1e-30,0.0],
            [f32::MAX,1.0,-f32::MAX], [3.0,4.0,0.0]].into_iter().enumerate() {
            sums.insert((17,23,[i as u32,0,0]),sum);
        }
        let original = sums.clone();
        let capacity = sums.capacity();
        finalize_vertex_normals(&mut sums);
        assert_eq!(sums.len(), original.len());
        assert_eq!(sums.capacity(), capacity);
        for (key, sum) in original {
            // This was evaluated separately at every shared vertex emission.
            let before = normal(sum);
            assert_eq!(sums[&key].map(f32::to_bits), before.map(f32::to_bits), "{key:?}");
        }
    }
}

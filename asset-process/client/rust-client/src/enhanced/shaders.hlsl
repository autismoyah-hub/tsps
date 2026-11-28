// Presentation-only world shader. Native geometry/animation is already resolved.
cbuffer Scene : register(b0) {
 float4 eyeNear; float4 cameraRight; float4 cameraDown; float4 cameraForward;
 float4 projection; // focal X/Y in NDC, far, exposure
 float4 lightDirection; float4 shadowCenter; float4 shadowRight; float4 shadowDown;
 float4 quality; // shadow resolution, AO samples, AA, tier
 float4 dimensions; // internal width/height, output width/height
};
Texture2D<float> shadowTexture : register(t0);
Texture2DArray<float4> materials : register(t1);
Texture2D<float4> sceneColor : register(t2);
Texture2D<float4> sceneNormalDepth : register(t3);
Texture2D<float4> classicBackground : register(t4);
SamplerState linearSampler : register(s0);
SamplerState pointSampler : register(s1);
struct Input {float3 p:POSITION;float3 n:NORMAL;float3 color:COLOR;float2 uv:TEXCOORD;float tex:TEXINDEX;float alpha:ALPHA;uint id:OBJECTID;};
// Alpha is one native face value, not a vertex-varying interpolant. Keeping it
// exact prevents nominally opaque faces acquiring tiny background contributions.
struct Surface {float4 p:SV_Position;float3 world:WORLD;float3 n:NORMAL;float3 color:COLOR;float2 uv:TEXCOORD;nointerpolation float tex:TEXINDEX;nointerpolation float alpha:ALPHA;nointerpolation uint id:OBJECTID;float depth:DEPTH;};
Surface worldVS(Input v){
 Surface o;float3 d=v.p-eyeNear.xyz;float3 c=float3(dot(d,cameraRight.xyz),dot(d,cameraDown.xyz),dot(d,cameraForward.xyz));
 float farZ=projection.z,nearZ=eyeNear.w;
 o.p=float4(c.x*projection.x+c.z*cameraRight.w,-c.y*projection.y-c.z*cameraDown.w,(c.z-nearZ)*farZ/(farZ-nearZ),c.z);
 o.world=v.p;o.n=v.n;o.color=v.color;o.uv=v.uv;o.tex=v.tex;o.alpha=v.alpha;o.id=v.id+(uint)lightDirection.w;o.depth=c.z;return o;
}
float3 shadowCoordinates(float3 p){
 float3 d=p-shadowCenter.xyz;float extent=shadowCenter.w;
 return float3(dot(d,shadowRight.xyz)/(extent*2)+0.5,dot(d,shadowDown.xyz)/(extent*2)+0.5,0.5-dot(d,lightDirection.xyz)/4096.0);
}
Surface shadowVS(Input v){Surface o=(Surface)0;float3 s=shadowCoordinates(v.p);o.p=float4(s.x*2-1,1-s.y*2,s.z,1);o.uv=v.uv;o.tex=v.tex;o.alpha=v.alpha;return o;}
void shadowPS(Surface i){if(i.alpha<0.5)discard;if(i.tex>=0&&materials.Sample(linearSampler,float3(i.uv,i.tex)).a<0.5)discard;}
float shadowAmount(float3 p,float3 n){
 if(quality.x<1)return 1;
 float3 s=shadowCoordinates(p+n*0.8);if(any(s.xy<0)||any(s.xy>1)||s.z<=0||s.z>=1)return 1;
 // A receiver-plane bias follows the world size of a shadow texel. A fixed
 // depth bias underestimates the slope across one texel and stripes broad,
 // thin fitted paving. Bound the bias to retain close contact shadows.
 float cosine=max(0.15,saturate(dot(n,lightDirection.xyz)));
 float slope=sqrt(max(0,1-cosine*cosine))/cosine;
 float texelWorld=shadowCenter.w*2/max(1,quality.x);
 float bias=clamp((1.6*texelWorld*slope+1.0)/4096.0,0.0005,0.0015);
 float result=0;int radius=quality.w>1?1:0;
 if(radius==0){
  [unroll]for(int j=0;j<4;j++){float2 off=float2((j&1)?0.5:-0.5,(j&2)?0.5:-0.5)/quality.x;result+=(s.z-bias<=shadowTexture.SampleLevel(pointSampler,s.xy+off,0))?1:0;}return result*0.25;
 }
 [unroll]for(int y=-1;y<=1;y++)[unroll]for(int x=-1;x<=1;x++)result+=(s.z-bias<=shadowTexture.SampleLevel(pointSampler,s.xy+float2(x,y)/quality.x,0))?1:0;
 return result/9;
}
struct Lit {float4 color:SV_Target0;float4 normalDepth:SV_Target1;uint id:SV_Target2;};
Lit worldPS(Surface i,bool front:SV_IsFrontFace){
 Lit o;float3 n=normalize(i.n);if(!front)n=-n;
 float3 albedo=i.color;
 if(i.tex>=0){float4 t=materials.Sample(linearSampler,float3(i.uv,i.tex));if(t.a<0.1)discard;albedo*=pow(max(t.rgb,0),2.2);}
 float hemi=saturate(0.5-0.5*n.y),key=max(0,dot(n,lightDirection.xyz));
 float shade=shadowAmount(i.world,n);
 float3 ambient=lerp(float3(0.22,0.205,0.18),float3(0.42,0.46,0.51),hemi);
 float fill=max(0,dot(n,normalize(float3(0.5,-0.3,0.7))))*0.15;
 float3 l=albedo*(ambient+fill+key*shade*0.9);
 if(quality.w>1){float3 v=normalize(eyeNear.xyz-i.world);float3 h=normalize(v+lightDirection.xyz);l+=pow(max(0,dot(n,h)),32)*0.035*shade;}
 o.color=float4(l,i.alpha);o.normalDepth=float4(n,i.depth);o.id=i.id;return o;
}
struct Quad {float4 p:SV_Position;float2 uv:TEXCOORD;};
Quad postVS(uint vertex:SV_VertexID){Quad o;o.uv=float2((vertex<<1)&2,vertex&2);o.p=float4(o.uv*float2(2,-2)+float2(-1,1),0,1);return o;}
float3 tone(float3 c){c*=projection.w;return pow(saturate((c*(2.51*c+0.03))/(c*(2.43*c+0.59)+0.14)),1.0/2.2);}
float3 surfacePosition(float2 uv,float depth){
 float2 ray=float2((uv.x*2-1-cameraRight.w)/projection.x,(uv.y*2-1-cameraDown.w)/projection.y);
 return eyeNear.xyz+depth*(cameraForward.xyz+cameraRight.xyz*ray.x+cameraDown.xyz*ray.y);
}
float4 postPS(Quad i):SV_Target {
 float2 px=1/dimensions.xy;float4 sample=sceneColor.SampleLevel(linearSampler,i.uv,0);float3 c=sample.rgb;float coverage=sample.a;
 float4 nd=sceneNormalDepth.SampleLevel(pointSampler,i.uv,0);
 if(nd.a>0&&quality.y>0){
  float ao=0;const float2 dirs[8]={float2(1,0),float2(-1,0),float2(0,1),float2(0,-1),float2(0.707,0.707),float2(-0.707,0.707),float2(0.707,-0.707),float2(-0.707,-0.707)};
  // Compare height above the actual local surface, not raw view depth. Raw
  // depth differences classify an ordinary sloped floor as an occluder and
  // turn half-float depth quantisation into horizontal material banding.
  float3 p=surfacePosition(i.uv,nd.a),n=normalize(nd.xyz);
  [loop]for(int k=0;k<(int)quality.y;k++){
   float radius=3+(k/8)*4;float2 uv=i.uv+dirs[k%8]*px*radius;
   float4 q=sceneNormalDepth.SampleLevel(pointSampler,uv,0);
   float3 delta=surfacePosition(uv,q.a)-p;float distance=length(delta),height=dot(delta,n);
   if(q.a>0&&distance>2&&distance<55&&height>max(1.5,nd.a*0.00075))ao+=saturate(height/distance)*(1-distance/55);
  }
  c*=1-0.35*ao/max(1,quality.y);
 }
 if(quality.z>0){float3 a=sceneColor.SampleLevel(linearSampler,i.uv+float2(px.x,0),0).rgb,b=sceneColor.SampleLevel(linearSampler,i.uv-float2(px.x,0),0).rgb,d=sceneColor.SampleLevel(linearSampler,i.uv+float2(0,px.y),0).rgb,e=sceneColor.SampleLevel(linearSampler,i.uv-float2(0,px.y),0).rgb;float edge=max(length(a-b),length(d-e));c=lerp(c,(a+b+d+e)*0.25,saturate((edge-0.12)*1.5)*0.35);}
 if(shadowRight.w>0){
  // World color is premultiplied over transparent black. Keep the completed
  // Classic sky unlit and at native resolution; it never casts or receives shadows.
  if(quality.z>0){
   float4 a=sceneColor.SampleLevel(linearSampler,i.uv+float2(px.x,0),0),b=sceneColor.SampleLevel(linearSampler,i.uv-float2(px.x,0),0),d=sceneColor.SampleLevel(linearSampler,i.uv+float2(0,px.y),0),e=sceneColor.SampleLevel(linearSampler,i.uv-float2(0,px.y),0);
   float edge=max(length(a.rgb-b.rgb),length(d.rgb-e.rgb));coverage=lerp(coverage,(a.a+b.a+d.a+e.a)*0.25,saturate((edge-0.12)*1.5)*0.35);
  }
  if(coverage>=1)return float4(tone(c),1);
  float3 sky=classicBackground.Load(int3(int2(i.p.xy),0)).rgb;
  return float4(lerp(sky,tone(c/max(coverage,0.00001)),saturate(coverage)),1);
 }
 return float4(tone(c),1);
}

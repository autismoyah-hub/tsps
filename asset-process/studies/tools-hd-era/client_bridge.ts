import fs from 'node:fs';
import path from 'node:path';
import {ByteBuffer} from '{TSPS_CHECKOUT}/client/rs/io/ByteBuffer';
import {ProceduralTextureLoader} from '{TSPS_CHECKOUT}/client/rs/texture/ProceduralTextureLoader';
import {computeTextureCoords} from '{TSPS_CHECKOUT}/client/rs/model/TextureMapper';

if(process.argv[2]==='decode' || process.argv[2]==='rawuv'){
    const {ModelData}=require('{TSPS_CHECKOUT}/client/rs/model/ModelData');
    const m=ModelData.decode(Int8Array.from(fs.readFileSync(0)));
    if(process.argv[2]==='rawuv'){
        const result=computeTextureCoords({isSd:()=>true} as any,m);
        process.stdout.write(Buffer.from(result!.buffer));
    }else{
    const rows=(a,b,c)=>Array.from(a,(x,i)=>[x,b[i],c[i]]);
    process.stdout.write(JSON.stringify({vertices:rows(m.verticesX,m.verticesY,m.verticesZ),faces:rows(m.indices1,m.indices2,m.indices3),colors:Array.from(m.faceColors,x=>x&65535)}));
    }
}else if(process.argv[2]==='uv'){
    const m=JSON.parse(fs.readFileSync(0,'utf8'));
    const uv=computeTextureCoords({isSd:()=>true} as any,m);
    process.stdout.write(Buffer.from(uv!.buffer));
}else{
    const root=process.argv[2],rev=Number(process.argv[3]);
    const inputs=JSON.parse(fs.readFileSync(path.join(root,'inputs.json'),'utf8'));
    const index=(records:Record<string,string>)=>{
        function file(id:number){
            if(!(id in records))return undefined;
            const data=Int8Array.from(Buffer.from(records[id],'base64'));
            return {data,getDataAsBuffer:()=>new ByteBuffer(data)};
        }
        return {getFileSmart:file,getArchiveIds:()=>Object.keys(records).map(Number),getFile:(g:number,f:number)=>f===0?file(g):undefined};
    };
    const loader=ProceduralTextureLoader.load(rev,index(inputs.materials) as any,index(inputs.textures) as any,index(inputs.sprites) as any);
    const errors={},meta={};
    for(const id of loader.getTextureIds()){
        try {
            const t=loader.getTexture(id)!;
            const pixels=loader.getPixelsArgb(id,128,false,1);
            const rgba=Buffer.alloc(pixels.length*4);
            for(let j=0;j<pixels.length;j++){
                rgba[j*4]=pixels[j]>>>16;rgba[j*4+1]=pixels[j]>>>8;
                rgba[j*4+2]=pixels[j];rgba[j*4+3]=pixels[j]>>>24;
            }
            fs.writeFileSync(path.join(root,id+'.rgba'),rgba);
            meta[id]={material:loader.materials[id],recipe:{sprites:t.proceduralTexture.spriteDependencies,textures:t.proceduralTexture.textureDependencies,operations:t.proceduralTexture.operations.map(op=>op.constructor.name)}};
        }catch(e){errors[id]=String(e);}
    }
    fs.writeFileSync(path.join(root,'meta.json'),JSON.stringify(meta));
    fs.writeFileSync(path.join(root,'errors.json'),JSON.stringify(errors));
}


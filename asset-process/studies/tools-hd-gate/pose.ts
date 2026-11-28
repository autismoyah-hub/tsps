import fs from 'node:fs';
import {ModelData} from '{TSPS_CHECKOUT}/client/rs/model/ModelData';
import {Model} from '{TSPS_CHECKOUT}/client/rs/model/Model';
import {Dat2SeqFrame} from '{TSPS_CHECKOUT}/client/rs/model/seq/SeqFrame';
import {Dat2SeqBase} from '{TSPS_CHECKOUT}/client/rs/model/seq/SeqBase';
const input=JSON.parse(fs.readFileSync(0,'utf8'));
const bytes=(s:string)=>Int8Array.from(Buffer.from(s,'base64'));
const info={game:'runescape',revision:input.rev} as any;
const bases={load:(id:number)=>Dat2SeqBase.load(info,id,bytes(input.bases[id]))};
const bind=ModelData.decode(bytes(input.model)).light({isSd:()=>true} as any,64,850,-30,-50,-30);
const result=input.frames.map((data:string)=>{
    const frame=Dat2SeqFrame.load(info,bases as any,bytes(data));
    const posed=Model.copyAnimated(bind,false,false);
    posed.animate(frame,undefined,input.op14);
    return {vertices:Array.from(posed.verticesX,(x,i)=>[x,posed.verticesY[i],posed.verticesZ[i]]),
            alpha:Array.from(posed.faceAlphas||[],x=>x&255),colors:Array.from(posed.faceColors||[],x=>x&65535)};
});
process.stdout.write(JSON.stringify(result));

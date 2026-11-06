import json,glob,sys,subprocess,re
root,bench,pos=sys.argv[1:4]
ME='honey_badger'
cases=[l.split(':') for l in sys.argv[4].split(',')]  # id:dec
for i,dec in cases:
    dec=int(dec)
    d=json.load(open(glob.glob(f'{root}/raw/duel_{i}*.json')[0]));fr={f['Turn']:f for f in d['frames']}
    print('#####',i,'letzte Entscheidung',dec)
    for T in range(dec-40,dec+1,8):
        f=fr[T]
        sn=[s for s in f['Snakes'] if not (s.get('Death') and s['Death']['Turn']<=T)]
        def api(s):
            b=[{'x':c['X'],'y':c['Y']} for c in s['Body']]
            return {'id':s['ID'],'name':s['Name'],'health':s['Health'],'body':b,'head':b[0],'length':len(b),'latency':'0','shout':''}
        ss=[api(s) for s in sn]
        me=[a for a in ss if a['name']==ME][0]
        st={'game':{'id':'x','ruleset':{'name':'standard','version':'1','settings':{'foodSpawnChance':15,'minimumFood':1,'hazardDamagePerTurn':14}},'map':'standard','timeout':500,'source':'arena'},'turn':T,'board':{'height':11,'width':11,'food':[{'x':c['X'],'y':c['Y']} for c in f['Food']],'hazards':[],'snakes':ss},'you':me}
        p=f'{pos}/{i}_{T}.json';json.dump(st,open(p,'w'))
        nxt=[s for s in fr[T+1]['Snakes'] if s['Name']==ME][0]['Body'][0]
        h=me['head'];dx,dy=nxt['X']-h['x'],nxt['Y']-h['y']
        played={(0,1):'up',(0,-1):'down',(-1,0):'left',(1,0):'right'}.get((dx,dy))
        res=[]
        for ms in (300,3000):
            o=subprocess.run([bench,p,str(ms)],capture_output=True,text=True).stdout
            res.append(o)
        print('T',T,'gespielt',played,'lang',me['length'],'|',re.sub(r'\s+',' ',res[0])[:170]);print('      3s:',re.sub(r'\s+',' ',res[1])[:170])

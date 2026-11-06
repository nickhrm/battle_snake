import json,glob,sys,random
root,out=sys.argv[1:3]
random.seed(5)
ME='honey_badger'
for b,n in (('duel',24),('standard',24)):
    fs=sorted(glob.glob(f'{root}/raw/{b}_*.json'));random.shuffle(fs);made=0
    for p in fs:
        if made>=n: break
        d=json.load(open(p));fr=d['frames']
        want=[t for t in (15,60,120,200,300) if t<len(fr)-15]
        if not want: continue
        T=random.choice(want);f=fr[T]
        sn=[s for s in f['Snakes'] if not s.get('Death')]
        if b=='standard' and len(sn)<3: continue
        if not any(s['Name']==ME for s in sn): continue
        def api(s):
            q=[{'x':c['X'],'y':c['Y']} for c in s['Body']]
            return {'id':s['ID'],'name':s['Name'],'health':s['Health'],'body':q,'head':q[0],'length':len(q),'latency':'0','shout':''}
        ss=[api(s) for s in sn];me=[a for a in ss if a['name']==ME][0]
        j={'game':{'id':'x','ruleset':{'name':'standard','version':'1','settings':{'foodSpawnChance':15,'minimumFood':1,'hazardDamagePerTurn':14}},'map':'standard','timeout':500,'source':'arena'},'turn':T,'board':{'height':11,'width':11,'food':[{'x':c['X'],'y':c['Y']} for c in f['Food']],'hazards':[],'snakes':ss},'you':me}
        json.dump(j,open(f'{out}/{b}_{made:02d}.json','w'));made+=1
    print(b,made)

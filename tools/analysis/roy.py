import json,sys,collections as C,statistics as st
root=sys.argv[1]
H=json.load(open(root+'/entries/hist.json'))['rows']['royale']
ME='honey_badger'; PROF='2026-11-05 20:30'
from collections import deque
pl=lambda s:1 if '1st' in s else 2 if '2nd' in s else 3 if '3rd' in s else 4 if '4th' in s else 0
G=[]
for r in H:
    d=json.load(open(f"{root}/raw/royale_{r['id']}.json"));fr=d['frames']
    if len(fr)<5: continue
    rec=dict(r=r,p=pl(r['pl']),fr=fr);G.append(rec)
print('Beispiel Ruleset',G[0]['fr'][0].keys() if 0 else json.load(open(f"{root}/raw/royale_{H[0]['id']}.json"))['meta']['Game']['Ruleset'])
def stats(g):
    fr=g['fr'];S=C.defaultdict(lambda:dict(hz=0,t=0,eat=0,prev=None,hp=[],ln=[]))
    first={}
    for f in fr:
        hz={(c['X'],c['Y']) for c in (f.get('Hazards') or [])}
        if hz and 'first' not in first: first['first']=f['Turn']
        for s in f['Snakes']:
            if s.get('Death'):continue
            x=S[s['Name']];h=(s['Body'][0]['X'],s['Body'][0]['Y'])
            x['t']+=1;x['hz']+=h in hz
            if x['prev'] is not None and len(s['Body'])>x['prev']: x['eat']+=1
            x['prev']=len(s['Body'])
    return S,first.get('first')
print('--- Epochen')
for name,sel in (('vor Profil',[g for g in G if g['r']['t']<PROF]),('Royale-Profil',[g for g in G if g['r']['t']>=PROF])):
    c=C.Counter();hz=[];tot=[];ot=[];oh=[]
    for g in sel:
        S,_=stats(g)
        m=S[ME];hz.append(m['hz']/max(1,m['t']));tot.append(m['t'])
        for n,x in S.items():
            if n!=ME: oh.append(x['hz']/max(1,x['t']))
        last=g['fr'][-1]['Snakes'];d=[s for s in last if s['Name']==ME][0].get('Death')
        c[d['Cause'] if d else 'lebt']+=1
    print(name,len(sel),'win',round(sum(g['p']==1 for g in sel)/len(sel),2),'place',round(st.mean(g['p'] or 2 for g in sel),2),'Hazard-Anteil Zuege wir',round(st.mean(hz),3),'Gegner',round(st.mean(oh),3),'Tod',c.most_common(6))
print('--- Royale-Tode durch hazard/hunger: Lage')
rows=[]
for g in G:
    last=g['fr'][-1]['Snakes'];d=[s for s in last if s['Name']==ME][0].get('Death')
    if not d or d['Cause'] not in('hazard','out-of-health'): continue
    T=d['Turn'];fm={f['Turn']:f for f in g['fr']}
    f=fm.get(T-1)
    if not f: continue
    hz={(c['X'],c['Y']) for c in (f.get('Hazards') or [])}
    me=[s for s in f['Snakes'] if s['Name']==ME][0]
    safe=121-len(hz)
    occ={(b['X'],b['Y']) for s in f['Snakes'] if not s.get('Death') for b in s['Body']}
    def bfs(start,goal):
        q=deque([(start,0)]);seen={start}
        while q:
            (x,y),dd=q.popleft()
            if dd>0 and (x,y) in goal: return dd
            for a,b in((x+1,y),(x-1,y),(x,y+1),(x,y-1)):
                if 0<=a<11 and 0<=b<11 and (a,b) not in seen and (a,b) not in occ:
                    seen.add((a,b));q.append(((a,b),dd+1))
        return None
    h=(me['Body'][0]['X'],me['Body'][0]['Y'])
    sf={(x,y) for x in range(11) for y in range(11)}-hz
    fd={(c['X'],c['Y']) for c in f['Food']}
    alive=[s for s in f['Snakes'] if not s.get('Death')]
    # ein paar Zuege vorher
    hps=[next((s['Health'] for s in fm[t]['Snakes'] if s['Name']==ME and not s.get('Death')),None) for t in (T-40,T-20,T-10,T-5) if t in fm]
    lastfood=None
    pv=None
    for t in range(T-1,0,-1):
        if t in fm and t-1 in fm:
            a=next((len(s['Body']) for s in fm[t]['Snakes'] if s['Name']==ME),0);b=next((len(s['Body']) for s in fm[t-1]['Snakes'] if s['Name']==ME),0)
            if a>b: lastfood=T-t;break
    rows.append(dict(cause=d['Cause'],T=T,safe=safe,dsafe=bfs(h,sf) if h in hz else 0,dfood=bfs(h,fd),nfood=len(fd),len=len(me['Body']),alive=len(alive),hps=hps,place=g['p'],lastfood=lastfood,oppmaxlen=max(len(s['Body']) for s in alive if s['Name']!=ME) if len(alive)>1 else 0,oppHP=[s['Health'] for s in alive if s['Name']!=ME],id=g['r']['id'][:8],t=g['r']['t'][5:16]))
for r in sorted(rows,key=lambda r:r['T']): print(r['t'],r['id'],r['cause'][:6],'T',r['T'],'Pl',r['place'],'safe',r['safe'],'dSafe',r['dsafe'],'dFood',r['dfood'],'nFood',r['nfood'],'len',r['len'],'oppLen',r['oppmaxlen'],'oppHP',r['oppHP'],'HP(-40,-20,-10,-5)',r['hps'],'seitFutter',r['lastfood'])
print('safe median',st.median(r['safe'] for r in rows),'T median',st.median(r['T'] for r in rows),'mit Futter erreichbar <=5:',sum(1 for r in rows if r['dfood'] and r['dfood']<=5),'von',len(rows),'safe==0:',sum(r['safe']==0 for r in rows),'safe<=5:',sum(r['safe']<=5 for r in rows))

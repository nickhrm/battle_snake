import json,re,glob,os,sys,time,urllib.request
from concurrent.futures import ThreadPoolExecutor
root=sys.argv[1]
ids=set()
for f in glob.glob(root+'/pages/*.html'):
    ids|=set(re.findall(r'/games/([a-f0-9-]{36})',open(f).read()))
UA={'User-Agent':'battlesnake-analysis'}
def get(u):
    for i in range(4):
        try:
            return json.load(urllib.request.urlopen(urllib.request.Request(u,headers=UA),timeout=30))
        except Exception as e:
            time.sleep(1+i*2)
    return None
def one(i):
    p=f'{root}/games/{i}.json'
    if os.path.exists(p): return
    meta=get(f'https://arena.battlesnake.com/api/games/{i}')
    if not meta or meta['Game'].get('Status')!='complete': return
    frames=[];off=0
    while True:
        d=get(f'https://arena.battlesnake.com/api/games/{i}/frames?offset={off}&limit=100')
        fs=(d or {}).get('Frames') or []
        if not fs: break
        for f in fs:
            frames.append({'T':f['Turn'],'S':[{'N':s['Name'],'A':s.get('Author'),'L':s.get('Latency'),'E':bool(s.get('Error')),'D':(s.get('Death') or {}).get('Cause'),'DT':(s.get('Death') or {}).get('Turn'),'H':s.get('Health'),'Len':len(s.get('Body') or [])} for s in f['Snakes']]})
        off+=len(fs)
        if len(fs)<100: break
    g=meta['Game']
    json.dump({'id':i,'created':int(g['Created']),'ruleset':g['RulesetName'],'map':g.get('Map'),'n':len(frames[0]['S']) if frames else 0,'frames':frames},open(p,'w'))
with ThreadPoolExecutor(4) as ex: list(ex.map(one,ids))
print(len(glob.glob(root+'/games/*.json')))

import json,sys,os,time,urllib.request
from concurrent.futures import ThreadPoolExecutor
root=sys.argv[1]; H=json.load(open(root+'/entries/hist.json'))['rows']
want=[]
for b,rs in H.items():
    for r in rs:
        if b=='duel' or (b in('constrictor',) ) or (b=='standard' and r['t']>='2026-11-05 10:50') or (b=='royale'):
            want.append((b,r['id']))
UA={'User-Agent':'battlesnake-analysis'}
def get(u):
    for i in range(4):
        try: return json.load(urllib.request.urlopen(urllib.request.Request(u,headers=UA),timeout=30))
        except Exception: time.sleep(1+2*i)
def one(x):
    b,i=x; p=f'{root}/raw/{b}_{i}.json'
    if os.path.exists(p): return
    m=get(f'https://arena.battlesnake.com/api/games/{i}')
    if not m or m['Game'].get('Status')!='complete': return
    fr=[];o=0
    while True:
        d=get(f'https://arena.battlesnake.com/api/games/{i}/frames?offset={o}&limit=100')
        fs=(d or {}).get('Frames') or []
        if not fs: break
        fr+=fs;o+=len(fs)
        if len(fs)<100: break
    json.dump({'meta':m,'frames':fr},open(p,'w'))
with ThreadPoolExecutor(4) as ex: list(ex.map(one,want))
print('FETCHDONE',len(os.listdir(root+'/raw')))

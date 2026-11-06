import json,glob,sys
root,ids=sys.argv[1],sys.argv[2].split(',')
ME='honey_badger'
for i in ids:
    p=glob.glob(f'{root}/raw/duel_{i}*.json')[0];d=json.load(open(p));fr=d['frames']
    T=fr[-1]['Turn'];last=fr[-1]['Snakes']
    m=[s for s in last if s['Name']==ME][0];o=[s for s in last if s['Name']!=ME][0]
    print('\n#####',i,o['Name'],'Ende',T,'Tod',m.get('Death'),'unser len',len(m['Body']),'HP',m['Health'],'Gegner len',len(o['Body']),'HP',o['Health'],'Gegner-Tod',o.get('Death'))
    print(' Latenz letzte 6:',[next(s['Latency'] for s in f['Snakes'] if s['Name']==ME) for f in fr[-6:]],'Fehler',[bool(next(s.get('Error') for s in f['Snakes'] if s['Name']==ME)) for f in fr[-6:]])
    for back in (6,1):
        if len(fr)<=back: continue
        f=fr[-1-back]
        g=[['.' for _ in range(11)] for _ in range(11)]
        for c in f['Food']: g[c['Y']][c['X']]='*'
        for s in f['Snakes']:
            if s.get('Death') and s['Death']['Turn']<f['Turn']: continue
            mine=s['Name']==ME
            for k,b in enumerate(s['Body']):
                g[b['Y']][b['X']]=('H' if mine else 'Y') if k==0 else ('o' if mine else 'x')
            t=s['Body'][-1];g[t['Y']][t['X']]='t' if mine else 'z'
        print(f" Zug {f['Turn']}  H=wir(kopf) t=schwanz, Y=gegner z=schwanz")
        print('\n'.join('   '+' '.join(r) for r in reversed(g)))

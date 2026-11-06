import json,glob,sys,re,collections as C,statistics as st
root=sys.argv[1]
H=json.load(open(root+'/entries/hist.json'))['rows']
FIX='2026-11-05 10:50'
TOP={'duel':{'Bwukinator','Odiodin','NMC²','rattler','AlphaSlinky'}}
ME='honey_badger'
pl=lambda s:1 if '1st' in s else 2 if '2nd' in s else 3 if '3rd' in s else 4 if '4th' in s else 0
out={}
for b in ['duel','standard','royale','constrictor']:
    rows=[r for r in H[b] if r['t']>=FIX]
    res=[]
    for r in rows:
        try:d=json.load(open(f"{root}/raw/{b}_{r['id']}.json"))
        except: continue
        fr=d['frames']; 
        if len(fr)<3: continue
        ser=C.defaultdict(dict); death={}
        for f in fr:
            for s in f['Snakes']:
                if s.get('Death'): death[s['Name']]=(s['Death']['Cause'],s['Death']['Turn'],s['Death'].get('EliminatedBy',''))
                ser[s['Name']][f['Turn']]=(len(s['Body']),s['Health'])
        names=dict((s['ID'],s['Name']) for s in fr[0]['Snakes'])
        dm=death.get(ME)
        kn=None
        if dm and dm[2]: kn=names.get(dm[2],dm[2])
        res.append(dict(r=r,p=pl(r['pl']),turns=fr[-1]['Turn'],dm=dm,killer=kn,ser=ser,top=r['opp'][0] in TOP.get(b,()) ))
    out[b]=res
    print('\n===',b,'n',len(res),'Platz',sorted(C.Counter(x['p'] for x in res).items()))
    L=[x for x in res if x['p']!=1]
    print(' Todesursachen (nicht gewonnen):',C.Counter((x['dm'] or ('ueberlebt',0,''))[0] for x in L).most_common())
    print(' Todeszug Median',st.median([x['dm'][1] for x in L if x['dm']]) if L else 0,'Quartile',[sorted(x['dm'][1] for x in L if x['dm'])[int(len([1 for y in L if y['dm']])*q)] for q in (.25,.75)])
    print(' Killer-Laenge groesser?')
    # Laengendifferenz an Stuetzzuegen
    for T in (25,50,100,150,200,300):
        rowd={}
        for name,sel in (('Sieg',[x for x in res if x['p']==1]),('Nied',L)):
            v=[];w=[]
            for x in sel:
                s=x['ser']
                if T in s[ME]:
                    o=[s[n][T][0] for n in s if n!=ME and T in s[n]]
                    if o: v.append(s[ME][T][0]-max(o)); w.append(s[ME][T][0])
            if v: rowd[name]=(len(v),round(st.mean(v),1),round(st.mean(w),1))
        print('  Zug',T,'(n, dLaenge zu laengstem Gegner, eigene Laenge):',rowd)
if True:
    res=out['duel']
    print('\n=== Duel Top5 vs Rest (gewonnen/verloren)')
    for name,sel in (('Top5',[x for x in res if x['top']]),('Rest-Sieg',[x for x in res if not x['top'] and x['p']==1]),('Rest-Nied',[x for x in res if not x['top'] and x['p']!=1])):
        print(name,len(sel),'Spiellaenge med',st.median(x['turns'] for x in sel),'Tod:',C.Counter((x['dm'] or ('-',))[0] for x in sel).most_common())
        for T in (25,50,100,150,200,300):
            v=[]
            for x in sel:
                s=x['ser']
                o=[n for n in s if n!=ME]
                if T in s[ME] and o and T in s[o[0]]: v.append(s[ME][T][0]-s[o[0]][T][0])
            if v: print('   T',T,'n',len(v),'dLen',round(st.mean(v),2),'vorne',round(sum(a>0 for a in v)/len(v),2))
    print('\n Top5-Spiele:')
    for x in [y for y in res if y['top']]:
        s=x['ser'];o=[n for n in s if n!=ME][0]
        lt=max(s[ME]); print(x['r']['t'],o,'Platz',x['p'],'Zuege',x['turns'],'Tod',x['dm'],'Len/HP Ende',s[ME][lt],'Gegner',s[o].get(lt),x['r']['id'][:8])

import glob,re,subprocess,sys,statistics as st
bin_,suite=sys.argv[1:3]
for b in ('duel','standard'):
    fs=sorted(glob.glob(f'{suite}/{b}_*.json'))[:14]
    for ms in (75,150,300,600,1200):
        D=[]
        for f in fs:
            o=subprocess.run([bin_,f,str(ms),'seq'],capture_output=True,text=True).stdout.strip().split('\n')[-1]
            r=re.search(r'depth=(\d+) score=(\S+) nodes=(\d+)',o)
            if r and int(r[1])>0 and abs(float(r[2]))<9e5: D.append(int(r[1]))
        print(b,'seq',ms,'ms n',len(D),'Tiefe',round(st.mean(D),2))

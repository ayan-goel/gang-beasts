"""Roof Crew: soft work suit, ridge helmet, goggles, gauntlets and safety boots."""
from mesh import SuitMesh

TEAM, INK, TAPE, STEEL, SOLE, GLASS, WHITE, BRASS = range(8)
COLORS = ['ed8863', '263945', 'f6dfaa', '657a82', '324d58', '344c5b', 'fff6d8', 'ffa348',
          'fff6d8', '263945', '263945', '263945', 'f6dfaa', 'ed8863', 'ed8863', 'ed8863']


def blend(first, second, weight):
    return [(first, 1-weight), (second, weight)]


def build():
    mesh = SuitMesh()
    torso = []
    for y, rx, rz, w, paint in [
        (.695,.095,.082,0,INK),(.727,.153,.124,0,INK),(.79,.173,.148,0,INK),
        (.865,.169,.147,.1,SOLE),(.899,.165,.144,.25,TEAM),(.95,.159,.144,.55,TEAM),
        (1.035,.176,.158,.9,TEAM),(1.11,.185,.165,1,TAPE),(1.137,.188,.165,1,TEAM),
        (1.213,.187,.15,1,TEAM),(1.258,.155,.125,1,INK),(1.284,.092,.092,1,INK)]:
        torso.append(((0,y,0),(rx,rz),blend('pelvis','chest',w),paint))
    mesh.rings(torso, 28)

    for sign, side in [(-1,'left'), (1,'right')]:
        hand, forearm, arm = [side+'_'+name for name in ['hand','forearm','upper_arm']]
        leg, shin, foot = [side+'_'+name for name in ['thigh','shin','foot']]
        rows = []
        for y,x,r,d,weights,paint in [
            (.586,.31,.026,.034,[(hand,1)],SOLE),(.605,.31,.063,.061,[(hand,1)],INK),
            (.654,.31,.078,.075,[(hand,1)],INK),(.69,.307,.072,.065,blend(hand,forearm,.18),STEEL),
            (.715,.306,.063,.061,blend(hand,forearm,.62),TAPE),(.735,.305,.06,.056,[(forearm,1)],INK),
            (.806,.300,.063,.063,[(forearm,1)],TEAM),(.907,.294,.065,.061,[(forearm,1)],TEAM),
            (.965,.29,.061,.06,blend(forearm,arm,.48),TEAM),(1.035,.282,.077,.072,[(arm,1)],TAPE),
            (1.058,.279,.079,.075,[(arm,1)],TEAM),(1.168,.262,.089,.085,[(arm,1)],TEAM),
            (1.221,.249,.081,.078,blend(arm,'chest',.12),TEAM),
            (1.255,.231,.042,.05,blend(arm,'chest',.32),TEAM)]:
            rows.append(((sign*x,y,0),(r,d),weights,paint))
        mesh.rings(rows)
        mesh.oval((sign*.26,.657,-.035),(.037,.049,.043),hand,INK)
        rows = []
        for y,z,r,d,weights,paint in [
            (.003,-.062,.065,.123,[(foot,1)],SOLE),(.018,-.065,.086,.15,[(foot,1)],SOLE),
            (.043,-.069,.092,.154,[(foot,1)],INK),(.081,-.076,.092,.143,[(foot,1)],INK),
            (.107,-.048,.081,.108,blend(foot,shin,.15),STEEL),
            (.125,-.016,.076,.085,blend(foot,shin,.55),INK),(.164,0,.076,.075,[(shin,1)],TAPE),
            (.184,0,.077,.076,[(shin,1)],INK),(.27,0,.078,.075,[(shin,1)],INK),
            (.343,0,.075,.074,[(shin,1)],STEEL),(.401,0,.071,.074,blend(shin,leg,.52),INK),
            (.463,0,.093,.094,[(leg,1)],INK),(.606,0,.105,.102,[(leg,1)],INK),
            (.698,0,.099,.095,blend(leg,'pelvis',.2),INK),
            (.751,0,.079,.082,blend(leg,'pelvis',.62),INK)]:
            rows.append(((sign*.108,y,z),(r,d),weights,paint))
        mesh.rings(rows)

    head = [('head',1)]
    def badge(side, count):
        # Four inlaid marks on both sides of the helmet identify a kit even in grayscale.
        sector = side % (count//2)
        return 8 + (sector - 1)//2 if sector in (1,3,5,7) else INK
    def crown(side, count):
        sector = side % (count//2)
        return 12 + sector//4 if sector%4 in (1,2) else TEAM
    rows = [
        (1.269,.085,.088,INK),(1.29,.131,.13,INK),(1.322,.163,.154,STEEL),
        (1.348,.178,.17,TEAM),(1.388,.183,.18,INK),(1.404,.187,.181,badge),
        (1.434,.188,.184,INK),(1.456,.19,.187,TAPE),(1.473,.193,.188,TEAM),
        (1.509,.181,.174,crown),(1.553,.145,.143,crown),(1.587,.084,.082,crown),
        (1.599,.023,.024,TEAM)]
    mesh.rings([((0,y,0),(r,d),head,c) for y,r,d,c in rows],32)
    mesh.oval((0,1.405,-.146),(.148,.058,.048),'head',INK,24)
    for sign in [-1,1]:
        mesh.oval((sign*.058,1.402,-.186),(.04,.033,.013),'head',WHITE)
        mesh.oval((sign*.054,1.399,-.198),(.015,.022,.007),'head',GLASS)
    mesh.oval((0,1.492,-.171),(.038,.032,.018),'head',BRASS)
    mesh.oval((0,1.493,-.184),(.025,.021,.009),'head',WHITE)
    return mesh

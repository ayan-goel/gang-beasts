"""Broadcast beans: squashy pear bodies, sugar eyes and striped wristbands."""
from mesh import SuitMesh

TEAM, INK, CREAM = 0, 1, 6
COLORS = ['ff533f','361c36','ffda94','bd7a76','703d56','432c49','fff8df','e49b51',
          'fff8df','361c36','361c36','361c36','ffda94','ff533f','ff533f','ff533f']


def blend(a, b, weight):
    return [(a, 1-weight), (b, weight)]


def build():
    mesh = SuitMesh()
    rows = []
    for y, rx, rz, w in [
        (.46,.076,.08,0),(.49,.151,.14,0),(.55,.224,.177,0),
        (.66,.265,.21,0),(.79,.274,.22,.04),(.91,.256,.215,.3),
        (1.02,.24,.197,.66),(1.12,.224,.181,.93),(1.20,.18,.147,1),
        (1.28,.132,.12,1)]:
        rows.append(((0,y,.006),(rx,rz),blend('pelvis','chest',w),TEAM))
    mesh.rings(rows,32)

    for sign, side in [(-1,'left'),(1,'right')]:
        hand, forearm, arm = [side+'_'+part for part in ('hand','forearm','upper_arm')]
        leg, shin, foot = [side+'_'+part for part in ('thigh','shin','foot')]
        def cuff(sector, count):
            return 12 + (sector * 4 // count)

        rows = []
        for y, x, r, weights, paint in [
            (.545,.348,.023,[(hand,1)],TEAM),(.56,.346,.064,[(hand,1)],TEAM),
            (.603,.344,.092,[(hand,1)],TEAM),(.662,.34,.098,[(hand,1)],TEAM),
            (.706,.336,.079,blend(hand,forearm,.25),INK),(.722,.335,.08,blend(hand,forearm,.4),cuff),
            (.747,.334,.078,blend(hand,forearm,.7),INK),(.762,.334,.071,[(forearm,1)],TEAM),
            (.855,.334,.072,[(forearm,1)],TEAM),(.963,.316,.081,blend(forearm,arm,.55),TEAM),
            (1.077,.287,.095,[(arm,1)],TEAM),(1.152,.249,.097,blend(arm,'chest',.18),TEAM),
            (1.20,.212,.066,blend(arm,'chest',.5),TEAM),(1.217,.191,.03,[('chest',1)],TEAM)]:
            rows.append(((sign*x,y,-.014),(r,r*.91),weights,paint))
        mesh.rings(rows)
        mesh.oval((sign*.28,.643,-.061),(.045,.058,.046),hand,TEAM)

        rows = []
        for y, x, z, rx, rz, weights in [
            (.012,.143,-.056,.053,.077,[(foot,1)]),(.041,.143,-.054,.077,.104,[(foot,1)]),
            (.108,.141,-.036,.09,.107,blend(foot,shin,.35)),(.20,.137,-.009,.093,.093,[(shin,1)]),
            (.33,.13,.006,.098,.102,[(shin,1)]),(.42,.127,.004,.106,.113,blend(shin,leg,.68)),
            (.54,.126,.006,.119,.132,[(leg,1)]),(.65,.119,.012,.126,.141,blend(leg,'pelvis',.36)),
            (.73,.112,.012,.11,.119,blend(leg,'pelvis',.7))]:
            rows.append(((sign*x,y,z),(rx,rz),weights,TEAM))
        mesh.rings(rows,24)

    mesh.oval((0,1.443,-.012),(.279,.293,.255),'head',TEAM,36)
    for sign in (-1,1):
        mesh.oval((sign*.108,1.473,-.243),(.065,.079,.035),'head',CREAM,24)
    return mesh

SuperStrict

Import BRL.StandardIO

Type Pressure
    Field L:Float
    Field E:Float
    Field C:Float
    Field P:Float
    Field I:Float
End Type

Type NodeState
    Field nodeId:Int
    Field volume:Double   ' m3
    Field inflow:Double   ' m3/s
    Field outflow:Double  ' m3/s
    Field storageMax:Double
    Field S:Double        ' species richness index
    Field press:Pressure
End Type

Function ExtinctionRate:Double( p:Pressure, alpha:Double[] )
    Local f:Double = alpha[0]*p.L + alpha[1]*p.E + alpha[2]*p.C + alpha[3]*p.P + alpha[4]*p.I
    Return f
End Function

Function StepNodeState( node:NodeState Var, dt:Double, k:Double, alpha:Double[] )
    ' Hydrological step: simple mass balance
    Local dV:Double = (node.inflow - node.outflow) * dt
    node.volume :+ dV
    If node.volume < 0 Then node.volume = 0
    If node.volume > node.storageMax Then node.volume = node.storageMax

    ' Biodiversity step
    Local f:Double = ExtinctionRate( node.press, alpha )
    Local dS:Double = -k * f * node.S * dt
    node.S :+ dS
    If node.S < 0 Then node.S = 0
End Function

Function RecommendGateOpen:Double( node:NodeState, floodThreshold:Double )
    ' Simple rule: open more if volume > threshold
    If node.volume > floodThreshold Then
        Return 1.0
    ElseIf node.volume > 0.8 * floodThreshold Then
        Return 0.5
    Else
        Return 0.0
    EndIf
End Function

' Main loop (pseudo-realtime)
Function Main()
    Local dt:Double = 60.0 ' 60 s
    Local k:Double = 1.0E-6
    Local alpha:Double[] = [0.2, 0.3, 0.2, 0.2, 0.1]

    Local node:NodeState = New NodeState
    node.nodeId = 1
    node.volume = 1.0e5
    node.storageMax = 2.0e5
    node.S = 100.0
    node.press = New Pressure
    node.press.L = 3.0
    node.press.E = 2.0
    node.press.C = 1.5
    node.press.P = 2.5
    node.press.I = 1.0

    Local floodThreshold:Double = 1.5e5

    While True
        ' TODO: read node.inflow, node.outflow, pressures from sensors/data stream

        StepNodeState( node, dt, k, alpha )
        Local gateOpen:Double = RecommendGateOpen( node, floodThreshold )

        Print "Volume=" + node.volume + " S=" + node.S + " Gate=" + gateOpen

        ' TODO: send gateOpen as recommendation to SCADA/human operator

        Delay 60000 ' 60 s
    Wend
End Function

Main()

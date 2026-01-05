SuperStrict

Import BRL.StandardIO
Import BRL.TextStream

Const MAX_NODES:Int = 64
Const MAX_GATES:Int = 64

Type Pressure
    Field L:Float
    Field E:Float
    Field C:Float
    Field P:Float
    Field I:Float
End Type

Type NodeState
    Field nodeId:Int
    Field name:String
    Field basinId:String
    Field nodeType:String
    Field lat:Double
    Field lon:Double
    Field elev:Double

    Field storageMax:Double
    Field volume:Double
    Field inflow:Double
    Field outflow:Double

    Field S:Double
    Field press:Pressure
End Type

Type GateState
    Field gateId:Int
    Field name:String
    Field fromNode:Int
    Field toNode:Int
    Field maxFlow:Double
    Field groupId:Int

    Field u:Double       ' 0..1
    Field flow:Double
End Type

Global Nodes:NodeState[MAX_NODES]
Global NodeCount:Int = 0
Global Gates:GateState[MAX_GATES]
Global GateCount:Int = 0

' ==== Math core ======================================================

Function ExtinctionRate:Double( p:Pressure, alpha:Double[] )
    Local f:Double = alpha[0]*p.L + alpha[1]*p.E + alpha[2]*p.C + alpha[3]*p.P + alpha[4]*p.I
    If f < 0.0 Then f = 0.0
    Return f
End Function

Function StepNode:Void( n:NodeState Var, dt:Double, k:Double, alpha:Double[] )
    Local dV:Double = (n.inflow - n.outflow) * dt
    n.volume :+ dV
    If n.volume < 0 Then n.volume = 0
    If n.volume > n.storageMax Then n.volume = n.storageMax

    Local f:Double = ExtinctionRate(n.press, alpha)
    Local dS:Double = -k * f * n.S * dt
    n.S :+ dS
    If n.S < 0 Then n.S = 0
End Function

Function RecommendGate:Double( g:GateState, floodThreshold:Double )
    Local fromNode:NodeState = FindNodeById(g.fromNode)
    If fromNode = Null Then Return 0.0

    If fromNode.volume > floodThreshold Then
        Return 1.0
    ElseIf fromNode.volume > 0.8 * floodThreshold Then
        Return 0.5
    Else
        Return 0.0
    EndIf
End Function

Function FindNodeById:NodeState( id:Int )
    For Local i:Int = 0 Until NodeCount
        If Nodes[i].nodeId = id Then Return Nodes[i]
    Next
    Return Null
End Function

Function FindNodeIndexById:Int( id:Int )
    For Local i:Int = 0 Until NodeCount
        If Nodes[i].nodeId = id Then Return i
    Next
    Return -1
End Function

' ==== CSV loading ====================================================

Function LoadNodes:Void( path:String )
    Local ts:TStream = ReadFile(path)
    If ts = Null Then
        Print "ERROR: cannot open nodes file: " + path
        End
    EndIf

    Local line:String = ts.ReadLine() ' header
    While Not ts.Eof()
        line = ts.ReadLine()
        If line = "" Then Continue

        Local fields:String[] = line.Split(",")
        If fields.Length < 15 Then Continue

        Local n:NodeState = Nodes[NodeCount]
        n.nodeId       = Int(fields[0].Trim())
        n.name         = fields[1].Trim()
        n.basinId      = fields[2].Trim()
        n.nodeType     = fields[3].Trim()
        n.lat          = Double(fields[4])
        n.lon          = Double(fields[5])
        n.elev         = Double(fields[6])
        n.storageMax   = Double(fields[7])
        n.volume       = Double(fields[8])
        n.S            = Double(fields[9])
        n.press        = New Pressure
        n.press.L      = Float(fields[10])
        n.press.E      = Float(fields[11])
        n.press.C      = Float(fields[12])
        n.press.P      = Float(fields[13])
        n.press.I      = Float(fields[14])

        Nodes[NodeCount] = n
        NodeCount :+ 1
        If NodeCount >= MAX_NODES Then Exit
    Wend

    ts.Close()
    Print "Loaded nodes: " + NodeCount
End Function

Function LoadGates:Void( path:String )
    Local ts:TStream = ReadFile(path)
    If ts = Null Then
        Print "ERROR: cannot open gates file: " + path
        End
    EndIf

    Local line:String = ts.ReadLine() ' header
    While Not ts.Eof()
        line = ts.ReadLine()
        If line = "" Then Continue

        Local fields:String[] = line.Split(",")
        If fields.Length < 6 Then Continue

        Local g:GateState = Gates[GateCount]
        g.gateId   = Int(fields[0].Trim())
        g.name     = fields[1].Trim()
        g.fromNode = Int(fields[2])
        g.toNode   = Int(fields[3])
        g.maxFlow  = Double(fields[4])
        g.groupId  = Int(fields[5])
        g.u        = 0.0
        g.flow     = 0.0

        Gates[GateCount] = g
        GateCount :+ 1
        If GateCount >= MAX_GATES Then Exit
    Wend

    ts.Close()
    Print "Loaded gates: " + GateCount
End Function

' ==== Simulation loop ================================================

Function UpdateGateFlows:Void()
    ' Compute gate flows and assign to node inflows/outflows
    ' For now, simple: outflow = sum over gates from node, inflow = sum to node
    ' Reset
    For Local i:Int = 0 Until NodeCount
        Nodes[i].inflow = 0
        Nodes[i].outflow = 0
    Next

    For Local gIdx:Int = 0 Until GateCount
        Local g:GateState = Gates[gIdx]
        g.flow = g.u * g.maxFlow

        Local fi:Int = FindNodeIndexById(g.fromNode)
        Local ti:Int = FindNodeIndexById(g.toNode)
        If fi >= 0 Then Nodes[fi].outflow :+ g.flow
        If ti >= 0 Then Nodes[ti].inflow :+ g.flow

        Gates[gIdx] = g
    Next
End Function

Function StepAllNodes:Void( dt:Double, k:Double, alpha:Double[] )
    For Local i:Int = 0 Until NodeCount
        StepNode(Nodes[i], dt, k, alpha)
    Next
End Function

Function RecommendAllGates:Void( floodThreshold:Double )
    For Local gIdx:Int = 0 Until GateCount
        Local g:GateState = Gates[gIdx]
        g.u = RecommendGate(g, floodThreshold)
        Gates[gIdx] = g
    Next
End Function

' Simple logging to stdout (can redirect to file)
Function LogState:Void( t:Double )
    For Local i:Int = 0 Until NodeCount
        Local n:NodeState = Nodes[i]
        Print t + "," + n.nodeId + "," + n.name + "," + n.volume + "," + n.S
    Next
End Function

' ==== Main ===========================================================

Function Main()
    Local nodesPath:String = "config/tempe_phoenix_nodes.csv"
    Local gatesPath:String = "config/tempe_phoenix_gates.csv"

    LoadNodes(nodesPath)
    LoadGates(gatesPath)

    Local dt:Double = 60.0  ' seconds
    Local k:Double = 1.0E-6
    Local alpha:Double[] = [0.2, 0.3, 0.2, 0.2, 0.1]
    Local floodThreshold:Double = 1.5e6  ' example threshold for Tempe Lake node

    Local t:Double = 0.0
    While t < 86400.0  ' simulate one day for testing
        RecommendAllGates(floodThreshold)
        UpdateGateFlows()
        StepAllNodes(dt, k, alpha)
        LogState(t)

        t :+ dt
        ' Delay 10 to throttle in test mode; in deployment, sync to real clock
        Delay(10)
    Wend
End Function

Main()

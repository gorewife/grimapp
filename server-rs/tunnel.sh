#!/bin/bash
# Start SSH tunnel to botserver database
# This creates a tunnel from localhost:5432 to the PostgreSQL server on botserver

echo "🚀 Starting SSH tunnel to botserver..."
echo "   Local:  localhost:5432"
echo "   Remote: botserver:5432 (grimkeeper_db)"
echo ""

# Kill any existing SSH tunnels to botserver on port 5432
if pgrep -f "ssh.*5432.*botserver" >/dev/null 2>&1 ; then
    echo "🔄 Existing SSH tunnel found, stopping it..."
    pkill -f "ssh.*5432.*botserver"
    sleep 1
    echo "✓  Old tunnel stopped"
    echo ""
fi

# Check if local PostgreSQL is running on port 5432
if lsof -Pi :5432 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo "⚠️  Local PostgreSQL is running on port 5432"
    echo "   Stopping it to free up the port..."
    brew services stop postgresql@15 2>/dev/null || true
    sleep 2
    echo "✓  Local PostgreSQL stopped"
    echo ""
fi

echo "Press Ctrl+C to stop the tunnel"
echo ""

ssh -N -L 5432:localhost:5432 botserver

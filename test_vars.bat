cd rewrite/rs-dash-pro
echo "Testing variable assignment..."

# First test directly
echo "Test 1: Direct echo"
.\target\release\rs-dash-pro.exe -c "echo test"

echo ""
echo "Test 2: Variable assignment and echo"
.\target\release\rs-dash-pro.exe -c "MYVAR=test; echo \$MYVAR"

echo ""
echo "Test 3: Just variable assignment"
.\target\release\rs-dash-pro.exe -c "MYVAR=test"

echo ""
echo "Test 4: Multiple assignments"
.\target\release\rs-dash-pro.exe -c "A=1 B=2; echo \$A \$B"

echo ""
echo "Test 5: Assignment with command"
.\target\release\rs-dash-pro.exe -c "PATH=/usr/bin echo hello"
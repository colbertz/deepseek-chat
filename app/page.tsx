'use client'

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import Navbar from '../components/Navbar';
import Conversation from '../components/Conversation';

export default function Home() {
  const [darkMode, setDarkMode] = useState(true);
  const [navOpen, setNavOpen] = useState(true);
  const [user, setUser] = useState(null);
  const router = useRouter();

  useEffect(() => {
    const checkAuth = async () => {
      try {
        const response = await fetch('/api/auth/me', {
          headers: {
            'Authorization': `Bearer ${document.cookie
              .split('; ')
              .find(row => row.startsWith('access_token='))
              ?.split('=')[1]}`
          }
        });

        if (!response.ok) {
          throw new Error('Unauthorized');
        }

        const userData = await response.json();
        setUser(userData);
      } catch (err) {
        router.push('/login');
      }
    };

    checkAuth();
  }, [router]);

  return (
    <div className={`flex h-screen ${darkMode ? 'bg-gray-900' : 'bg-gray-100'}`}>
      {/* Left Sidebar */}
      {navOpen && <Navbar darkMode={darkMode} setNavOpen={setNavOpen} />}

      {/* Main Content Area */}
      <Conversation 
        darkMode={darkMode}
        navOpen={navOpen}
        setDarkMode={setDarkMode}
        setNavOpen={setNavOpen}
      />
    </div>
  );
}

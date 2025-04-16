import { NextResponse } from 'next/server'
import { User } from '../types'
import { API_BASE_URL } from '@/config/api'
import { cookies } from 'next/headers'

export async function GET(request: Request) {
  try {
    const cookieStore = await cookies()
    let token = cookieStore.get('access_token')?.value
    
    if (!token) {
      token = request.headers.get('Authorization')?.split(' ')[1]
      if (!token) {
        return NextResponse.json({ error: 'Missing token' }, { status: 401 })
      }
    }

    const response = await fetch(`${API_BASE_URL}/auth/me`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (!response.ok) {
      const error = await response.json()
      return NextResponse.json({ error: error.error }, { status: response.status })
    }

    const user: User = await response.json()
    return NextResponse.json(user)
  } catch (err) {
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}

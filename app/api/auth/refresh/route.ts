import { NextResponse } from 'next/server'
import { RefreshRequest, AuthResponse } from '../types'
import { API_BASE_URL } from '@/config/api'
import { cookies } from 'next/headers'

export async function POST(request: Request) {
  try {
    const body: RefreshRequest = await request.json()
    const cookieStore = await cookies()
    const accessToken = cookieStore.get('access_token')?.value
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    }
    if (accessToken) {
      headers['Authorization'] = `Bearer ${accessToken}`
    }

    const response = await fetch(`${API_BASE_URL}/auth/refresh`, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    })

    if (!response.ok) {
      const error = await response.json()
      return NextResponse.json({ error: error.error }, { status: response.status })
    }

    const data: AuthResponse = await response.json()
    return NextResponse.json(data)
  } catch (err) {
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}

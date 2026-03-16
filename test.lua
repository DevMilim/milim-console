player = {
    move = function(self,x,y)
        print(x,y)
        print(self.data)
    end,
    data = 1,
    add = function(self)
        self.data = self.data + 1
    end
}

player:move(10,20)
player:add()
player:move(10,20)